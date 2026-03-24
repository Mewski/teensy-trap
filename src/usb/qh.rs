// EHCI link pointer type field (bits 2:1)
const QH_TYPE: u32 = 0x02; // 01 = Queue Head
const TERMINATE: u32 = 0x01; // T bit

#[repr(C, align(64))]
pub struct QueueHead {
  pub horizontal_link: u32,
  pub characteristics: u32,
  pub capabilities: u32,
  pub current_qtd: u32,
  pub next_qtd: u32,
  pub alt_next_qtd: u32,
  pub token: u32,
  pub buffer_ptrs: [u32; 5],
}

impl QueueHead {
  pub const fn empty() -> Self {
    Self {
      horizontal_link: TERMINATE,
      characteristics: 0,
      capabilities: 0,
      current_qtd: 0,
      next_qtd: TERMINATE,
      alt_next_qtd: TERMINATE,
      token: 0,
      buffer_ptrs: [0; 5],
    }
  }

  pub fn link_next_qh(&mut self, next: &QueueHead) {
    let addr = (next as *const QueueHead) as u32;
    self.horizontal_link = (addr & !0x1F) | QH_TYPE;
  }

  pub fn link_self_circular(&mut self) {
    let addr = (self as *const QueueHead) as u32;
    // H bit (bit 15 of characteristics) marks this as the head of the async schedule.
    // Horizontal link points back to self with QH type.
    self.horizontal_link = (addr & !0x1F) | QH_TYPE;
    self.characteristics |= 1 << 15;
  }

  pub fn link_qtd(&mut self, qtd_addr: u32) {
    self.next_qtd = qtd_addr & !0x1F;
  }

  pub fn set_device_addr(&mut self, addr: u8) {
    self.characteristics = (self.characteristics & !0x7F) | (addr as u32 & 0x7F);
  }

  pub fn set_endpoint(&mut self, ep: u8) {
    self.characteristics = (self.characteristics & !0x0F00) | ((ep as u32 & 0x0F) << 8);
  }

  pub fn set_max_packet_size(&mut self, size: u16) {
    self.characteristics = (self.characteristics & !0x07FF_0000) | ((size as u32 & 0x7FF) << 16);
  }

  pub fn set_speed(&mut self, speed: EndpointSpeed) {
    let val = match speed {
      EndpointSpeed::Full => 0,
      EndpointSpeed::Low => 1,
      EndpointSpeed::High => 2,
    };
    self.characteristics = (self.characteristics & !0x3000) | (val << 12);
  }

  pub fn set_data_toggle_from_qtd(&mut self) {
    // DTC bit (14): when set, data toggle comes from the qTD, not the QH
    self.characteristics |= 1 << 14;
  }

  pub fn set_control_endpoint_flag(&mut self) {
    // C bit (27): must be set for control endpoints on full/low speed devices.
    // Not needed for high-speed.
    self.characteristics |= 1 << 27;
  }

  pub fn set_nak_reload(&mut self, count: u8) {
    self.characteristics = (self.characteristics & !0xF000_0000) | ((count as u32 & 0x0F) << 28);
  }

  pub fn set_interrupt_schedule_mask(&mut self, s_mask: u8) {
    // S-mask (bits 7:0 of capabilities): microframe schedule mask.
    // Each bit enables polling in that microframe (0-7) within each frame.
    // 0xFF = every microframe = 8kHz for HS interrupt endpoints.
    // Must match the upstream device's bInterval capability.
    self.capabilities = (self.capabilities & !0xFF) | s_mask as u32;
  }

  pub fn set_split_completion_mask(&mut self, c_mask: u8) {
    // C-mask (bits 15:8 of capabilities): used for split transactions (FS/LS behind hub).
    // Not needed for direct HS connections.
    self.capabilities = (self.capabilities & !0xFF00) | ((c_mask as u32) << 8);
  }

  pub fn set_hub_info(&mut self, hub_addr: u8, port_num: u8) {
    // For split transactions with FS/LS devices behind a HS hub.
    // Hub Addr: bits 22:16, Port Number: bits 29:23
    self.capabilities =
      (self.capabilities & !0x3FFF_0000) | ((hub_addr as u32 & 0x7F) << 16) | ((port_num as u32 & 0x7F) << 23);
  }

  pub fn set_mult(&mut self, mult: u8) {
    // High-bandwidth pipe multiplier (bits 31:30). For regular endpoints, set to 1.
    self.capabilities = (self.capabilities & !0xC000_0000) | ((mult as u32 & 0x03) << 30);
  }

  pub fn configure_for_control(&mut self, addr: u8, max_packet_size: u16, speed: EndpointSpeed) {
    self.set_device_addr(addr);
    self.set_endpoint(0);
    self.set_max_packet_size(max_packet_size);
    self.set_speed(speed);
    self.set_data_toggle_from_qtd();
    self.set_nak_reload(15);
    self.set_mult(1);
    self.link_self_circular();
  }

  pub fn configure_for_interrupt(
    &mut self,
    addr: u8,
    endpoint: u8,
    max_packet_size: u16,
    speed: EndpointSpeed,
    s_mask: u8,
  ) {
    self.set_device_addr(addr);
    self.set_endpoint(endpoint);
    self.set_max_packet_size(max_packet_size);
    self.set_speed(speed);
    self.set_data_toggle_from_qtd();
    self.set_nak_reload(0); // no NAK reload for interrupt QHs
    self.set_mult(1);
    self.set_interrupt_schedule_mask(s_mask);
    self.horizontal_link = TERMINATE;
  }
}

pub enum EndpointSpeed {
  Full,
  Low,
  High,
}

/// Compute the S-mask for an interrupt endpoint based on its bInterval.
/// For HS endpoints, bInterval is in 125µs units (2^(bInterval-1) microframes).
/// Returns the appropriate S-mask byte.
pub fn compute_interrupt_smask(interval: u8, is_high_speed: bool) -> u8 {
  if !is_high_speed {
    // Full/low speed: poll once per frame, microframe 0
    return 0x01;
  }
  // High-speed: bInterval = 1 means every microframe (8kHz),
  // bInterval = 2 means every 2nd, bInterval = 3 means every 4th, etc.
  // Period in microframes = 2^(bInterval - 1), clamped to 1..8
  let period = if interval == 0 {
    1
  } else {
    1u8.wrapping_shl((interval - 1) as u32)
  };
  match period {
    1 => 0xFF, // every microframe (8kHz)
    2 => 0x55, // microframes 0, 2, 4, 6 (4kHz)
    4 => 0x11, // microframes 0, 4 (2kHz)
    _ => 0x01, // microframe 0 only (1kHz or slower)
  }
}
