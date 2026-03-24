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
      horizontal_link: 1,
      characteristics: 0,
      capabilities: 0,
      current_qtd: 0,
      next_qtd: 1,
      alt_next_qtd: 1,
      token: 0,
      buffer_ptrs: [0; 5],
    }
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

  pub fn set_high_speed(&mut self) {
    self.characteristics = (self.characteristics & !0x3000) | (2 << 12);
  }

  pub fn set_interrupt_schedule_mask(&mut self, s_mask: u8) {
    self.capabilities = (self.capabilities & !0xFF) | s_mask as u32;
  }
}
