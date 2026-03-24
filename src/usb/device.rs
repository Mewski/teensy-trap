use imxrt_ral as ral;

const MAX_ENDPOINTS: usize = 8;
const DQH_COUNT: usize = MAX_ENDPOINTS * 2; // IN + OUT for each

#[repr(C, align(2048))]
pub struct DqhList {
  pub entries: [DeviceQueueHead; DQH_COUNT],
}

impl DqhList {
  pub const fn new() -> Self {
    const EMPTY: DeviceQueueHead = DeviceQueueHead::empty();
    Self {
      entries: [EMPTY; DQH_COUNT],
    }
  }

  pub fn base_addr(&self) -> u32 {
    self.entries.as_ptr() as u32
  }

  // dQH index: OUT endpoints at even indices, IN at odd
  // EP0 OUT = 0, EP0 IN = 1, EP1 OUT = 2, EP1 IN = 3, etc.
  pub fn get_mut(&mut self, endpoint: u8, direction_in: bool) -> &mut DeviceQueueHead {
    let idx = (endpoint as usize) * 2 + if direction_in { 1 } else { 0 };
    &mut self.entries[idx]
  }
}

#[repr(C, align(64))]
pub struct DeviceQueueHead {
  pub capabilities: u32,
  pub current_dtd: u32,
  pub next_dtd: u32,
  pub token: u32,
  pub buffer_ptrs: [u32; 5],
  _reserved: u32,
  pub setup_buf: [u8; 8],
  _padding: [u32; 4],
}

impl DeviceQueueHead {
  pub const fn empty() -> Self {
    Self {
      capabilities: 0,
      current_dtd: 0,
      next_dtd: 1, // T-bit set = no DTD linked
      token: 0,
      buffer_ptrs: [0; 5],
      _reserved: 0,
      setup_buf: [0; 8],
      _padding: [0; 4],
    }
  }

  pub fn set_max_packet_size(&mut self, size: u16) {
    self.capabilities = (self.capabilities & !0x07FF_0000) | ((size as u32 & 0x7FF) << 16);
  }

  pub fn set_zero_length_termination(&mut self, disable: bool) {
    if disable {
      self.capabilities |= 1 << 29;
    } else {
      self.capabilities &= !(1 << 29);
    }
  }

  pub fn set_interrupt_on_setup(&mut self) {
    self.capabilities |= 1 << 15;
  }
}

#[repr(C, align(32))]
pub struct DeviceTransferDescriptor {
  pub next_dtd: u32,
  pub token: u32,
  pub buffer_ptrs: [u32; 5],
}

impl DeviceTransferDescriptor {
  pub const fn empty() -> Self {
    Self {
      next_dtd: 1, // T-bit set
      token: 0,
      buffer_ptrs: [0; 5],
    }
  }

  pub fn init(&mut self, buf: *const u8, len: u16, ioc: bool) {
    self.next_dtd = 1;
    self.token = ((len as u32 & 0x7FFF) << 16) | (1 << 7); // total_bytes + ACTIVE
    if ioc {
      self.token |= 1 << 15;
    }
    self.buffer_ptrs[0] = buf as u32;
    // Set page pointers for buffers crossing 4K boundaries
    let base = buf as u32;
    for i in 1..5 {
      self.buffer_ptrs[i] = (base & 0xFFFF_F000) + (i as u32 * 4096);
    }
  }

  pub fn is_active(&self) -> bool {
    self.token & (1 << 7) != 0
  }

  pub fn is_halted(&self) -> bool {
    self.token & (1 << 6) != 0
  }

  pub fn bytes_remaining(&self) -> u16 {
    ((self.token >> 16) & 0x7FFF) as u16
  }
}

pub struct DeviceController<const N: u8> {
  usb: ral::usb::Instance<N>,
}

impl<const N: u8> DeviceController<N> {
  pub fn new(usb: ral::usb::Instance<N>) -> Self {
    Self { usb }
  }

  pub fn reset(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg | (1 << 1));
    while ral::read_reg!(ral::usb, self.usb, USBCMD) & (1 << 1) != 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn set_device_mode(&mut self) {
    // CM = 0b10 (Device Controller), SLOM = 1 (Setup Lockout Mode off)
    ral::modify_reg!(ral::usb, self.usb, USBMODE, |reg| (reg & !0x03) | 0x02 | (1 << 3));
  }

  pub fn set_endpoint_list_addr(&mut self, addr: u32) {
    // In device mode, ASYNCLISTADDR is the endpoint list address
    ral::write_reg!(ral::usb, self.usb, ASYNCLISTADDR, addr & 0xFFFF_F800);
  }

  pub fn set_address(&mut self, addr: u8) {
    // USBADR bits 31:25, USBADRA bit 24 (advance address after status stage)
    let val = ((addr as u32) << 25) | (1 << 24);
    ral::write_reg!(ral::usb, self.usb, DEVICEADDR, val);
  }

  pub fn configure_endpoint(&mut self, ep_num: u8, ep_type: u8, tx: bool, rx: bool) {
    if ep_num == 0 {
      return; // EP0 is always control, configured by hardware
    }
    let idx = (ep_num - 1) as usize;
    if idx >= 7 {
      return;
    }
    let mut val = 0u32;
    if tx {
      val |= (1 << 23) | ((ep_type as u32 & 0x03) << 18); // TXE + TXT
    }
    if rx {
      val |= (1 << 7) | ((ep_type as u32 & 0x03) << 2); // RXE + RXT
    }
    // Set data toggle reset bits
    if tx {
      val |= 1 << 22; // TXR
    }
    if rx {
      val |= 1 << 6; // RXR
    }
    ral::write_reg!(ral::usb, self.usb, ENDPTCTRL[idx], val);
  }

  pub fn prime_endpoint_rx(&mut self, ep_num: u8) {
    // PERB (Prime Endpoint Receive Buffer) — bits 7:0
    ral::write_reg!(ral::usb, self.usb, ENDPTPRIME, 1u32 << ep_num);
  }

  pub fn prime_endpoint_tx(&mut self, ep_num: u8) {
    // PETB (Prime Endpoint Transmit Buffer) — bits 23:16
    ral::write_reg!(ral::usb, self.usb, ENDPTPRIME, 1u32 << (ep_num + 16));
  }

  pub fn flush_endpoint(&mut self, ep_num: u8, direction_in: bool) {
    let bit = if direction_in {
      1u32 << (ep_num + 16)
    } else {
      1u32 << ep_num
    };
    ral::write_reg!(ral::usb, self.usb, ENDPTFLUSH, bit);
    while ral::read_reg!(ral::usb, self.usb, ENDPTFLUSH) & bit != 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn stall_endpoint(&mut self, ep_num: u8, direction_in: bool) {
    if ep_num == 0 {
      // Stall both directions on EP0
      ral::modify_reg!(ral::usb, self.usb, ENDPTCTRL0, |reg| reg | (1 << 0) | (1 << 16));
    } else {
      let idx = (ep_num - 1) as usize;
      if idx >= 7 {
        return;
      }
      let stall_bit = if direction_in { 1u32 << 16 } else { 1u32 << 0 };
      ral::modify_reg!(ral::usb, self.usb, ENDPTCTRL[idx], |reg| reg | stall_bit);
    }
  }

  pub fn has_setup_packet(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, ENDPTSETUPSTAT) & 1 != 0
  }

  pub fn clear_setup_status(&mut self) {
    ral::write_reg!(ral::usb, self.usb, ENDPTSETUPSTAT, 1);
  }

  /// Read SETUP packet safely using the tripwire mechanism.
  /// Caller must pass the EP0 OUT dQH so we can read setup_buf.
  /// Returns None if no SETUP is pending.
  pub fn read_setup_packet(&mut self, ep0_out_dqh: &DeviceQueueHead) -> Option<[u8; 8]> {
    if !self.has_setup_packet() {
      return None;
    }
    // Use the setup tripwire to safely read 8 bytes from dQH.setup_buf.
    // If a new SETUP arrives during the read, SUTW is cleared and we retry.
    loop {
      self.setup_tripwire_begin();
      let data = ep0_out_dqh.setup_buf;
      if self.setup_tripwire_check() {
        self.setup_tripwire_clear();
        self.clear_setup_status();
        return Some(data);
      }
      // New SETUP arrived during read — retry
    }
  }

  pub fn is_port_connected(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, PORTSC1) & (1 << 0) != 0
  }

  pub fn bus_speed(&self) -> BusSpeed {
    match (ral::read_reg!(ral::usb, self.usb, PORTSC1) >> 26) & 0x03 {
      0 => BusSpeed::Full,
      2 => BusSpeed::High,
      _ => BusSpeed::Full,
    }
  }

  pub fn run(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg | (1 << 0));
  }

  pub fn stop(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg & !(1 << 0));
  }

  pub fn enable_interrupts(&mut self) {
    // UI, UEI, PCI, URI (USB Reset), SLI (Suspend)
    ral::write_reg!(
      ral::usb,
      self.usb,
      USBINTR,
      (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6) | (1 << 8)
    );
  }

  pub fn clear_all_status(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 0xFFFF_FFFF);
  }

  pub fn poll_usb_reset(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 6) != 0
  }

  pub fn ack_usb_reset(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 1 << 6);
  }

  pub fn poll_transfer_complete(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 0) != 0
  }

  pub fn ack_transfer_complete(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 1 << 0);
  }

  pub fn endpoint_complete(&self) -> u32 {
    let val = ral::read_reg!(ral::usb, self.usb, ENDPTCOMPLETE);
    ral::write_reg!(ral::usb, self.usb, ENDPTCOMPLETE, val);
    val
  }

  pub fn setup_tripwire_begin(&mut self) {
    // Set SUTW bit in USBCMD to safely read setup data
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg | (1 << 13));
  }

  pub fn setup_tripwire_check(&self) -> bool {
    // Check if SUTW is still set — if cleared, a new SETUP arrived during read
    ral::read_reg!(ral::usb, self.usb, USBCMD) & (1 << 13) != 0
  }

  pub fn setup_tripwire_clear(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg & !(1 << 13));
  }
}

#[derive(Debug, defmt::Format)]
pub enum BusSpeed {
  Full,
  High,
}
