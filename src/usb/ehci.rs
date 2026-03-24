use imxrt_ral as ral;

// PORTSC1 bits that are write-1-to-clear. Must be masked out during
// read-modify-write to avoid accidentally clearing status bits.
const PORTSC_W1C: u32 = (1 << 1) | (1 << 3) | (1 << 5); // CSC, PEC, OCC

fn portsc_rmw(reg: u32, set: u32, clear: u32) -> u32 {
  (reg & !PORTSC_W1C & !clear) | set
}

pub struct EhciHost<const N: u8> {
  usb: ral::usb::Instance<N>,
}

impl<const N: u8> EhciHost<N> {
  pub fn new(usb: ral::usb::Instance<N>) -> Self {
    Self { usb }
  }

  pub fn reset(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg | (1 << 1));
    while ral::read_reg!(ral::usb, self.usb, USBCMD) & (1 << 1) != 0 {
      cortex_m::asm::nop();
    }
    // TODO(hw): USBMODE must be set within ~2ms after reset on NXP Chipidea silicon
  }

  pub fn set_host_mode(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBMODE, |reg| (reg & !0x03) | 0x03);
  }

  pub fn detect_device(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, PORTSC1) & (1 << 0) != 0
  }

  pub fn port_speed(&self) -> PortSpeed {
    match (ral::read_reg!(ral::usb, self.usb, PORTSC1) >> 26) & 0x03 {
      0 => PortSpeed::Full,
      1 => PortSpeed::Low,
      2 => PortSpeed::High,
      _ => PortSpeed::Full,
    }
  }

  pub fn port_reset(&mut self) {
    // Set PR (Port Reset) bit 8, mask W1C bits
    ral::modify_reg!(ral::usb, self.usb, PORTSC1, |reg| portsc_rmw(reg, 1 << 8, 0));
    // USB 2.0 spec: root hub port reset requires at least 50ms
    cortex_m::asm::delay(30_000_000); // ~50ms at 600MHz
    // Clear PR bit, mask W1C bits
    ral::modify_reg!(ral::usb, self.usb, PORTSC1, |reg| portsc_rmw(reg, 0, 1 << 8));
    // Wait for PR to clear (hardware acknowledges)
    while ral::read_reg!(ral::usb, self.usb, PORTSC1) & (1 << 8) != 0 {
      cortex_m::asm::nop();
    }
    // TODO(hw): verify PE (bit 2) is set after reset — if not, device failed HS handshake
  }

  pub fn port_power_on(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, PORTSC1, |reg| portsc_rmw(reg, 1 << 12, 0));
  }

  pub fn is_port_enabled(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, PORTSC1) & (1 << 2) != 0
  }

  pub fn wait_for_connection(&self) {
    while !self.detect_device() {
      cortex_m::asm::nop();
    }
  }

  pub fn set_periodic_list_base(&mut self, addr: u32) {
    // In host mode, DEVICEADDR register offset (0x154) is PERIODICLISTBASE.
    // Must be 4KB aligned (EHCI spec Section 2.3.7).
    ral::write_reg!(ral::usb, self.usb, DEVICEADDR, addr & 0xFFFF_F000);
  }

  pub fn set_async_list_addr(&mut self, addr: u32) {
    // Must be 32-byte aligned (EHCI spec: points to a QH).
    ral::write_reg!(ral::usb, self.usb, ASYNCLISTADDR, addr & 0xFFFF_FFE0);
  }

  pub fn enable_periodic_schedule(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg | (1 << 4));
    while ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 14) == 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn disable_periodic_schedule(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg & !(1 << 4));
    while ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 14) != 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn enable_async_schedule(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg | (1 << 5));
    while ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 15) == 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn disable_async_schedule(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg & !(1 << 5));
    while ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 15) != 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn run(&mut self) {
    // ITC = 0 (immediate interrupt threshold) for lowest latency
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| (reg & !(0xFF << 16)) | (1 << 0));
  }

  pub fn stop(&mut self) {
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| reg & !(1 << 0));
    while ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 12) == 0 {
      cortex_m::asm::nop();
    }
  }

  pub fn enable_interrupts(&mut self) {
    // UI (transfer complete), UEI (error), PCI (port change), SEI (system error)
    ral::write_reg!(ral::usb, self.usb, USBINTR, (1 << 0) | (1 << 1) | (1 << 2) | (1 << 4));
  }

  pub fn clear_all_status(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 0xFFFF_FFFF);
  }

  pub fn poll_transfer_complete(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 0) != 0
  }

  pub fn ack_transfer_complete(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 1 << 0);
  }

  pub fn poll_port_change(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 2) != 0
  }

  pub fn ack_port_change(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 1 << 2);
  }

  pub fn poll_error(&self) -> bool {
    ral::read_reg!(ral::usb, self.usb, USBSTS) & (1 << 1) != 0
  }

  pub fn ack_error(&mut self) {
    ral::write_reg!(ral::usb, self.usb, USBSTS, 1 << 1);
  }

  pub fn frame_index(&self) -> u16 {
    (ral::read_reg!(ral::usb, self.usb, FRINDEX) & 0x3FFF) as u16
  }

  pub fn set_frame_list_size(&mut self, size: FrameListSize) {
    let fs = match size {
      FrameListSize::S1024 => 0b000,
      FrameListSize::S512 => 0b001,
      FrameListSize::S256 => 0b010,
      FrameListSize::S128 => 0b011,
      FrameListSize::S64 => 0b100,
      FrameListSize::S32 => 0b101,
      FrameListSize::S16 => 0b110,
      FrameListSize::S8 => 0b111,
    };
    let fs_low = fs & 0x03;
    let fs_high = (fs >> 2) & 0x01;
    ral::modify_reg!(ral::usb, self.usb, USBCMD, |reg| {
      (reg & !(0x03 << 2) & !(1 << 15)) | (fs_low << 2) | (fs_high << 15)
    });
  }
}

#[derive(Debug, defmt::Format)]
pub enum PortSpeed {
  Full,
  Low,
  High,
}

#[derive(Debug)]
pub enum FrameListSize {
  S1024,
  S512,
  S256,
  S128,
  S64,
  S32,
  S16,
  S8,
}
