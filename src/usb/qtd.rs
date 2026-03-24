use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct QtdToken: u32 {
        const PING_STATE    = 0x0000_0001;
        const SPLIT_STATE   = 0x0000_0002;
        const MISSED_UFRAME = 0x0000_0004;
        const XACT_ERR      = 0x0000_0008;
        const BABBLE        = 0x0000_0010;
        const DATA_BUF_ERR  = 0x0000_0020;
        const HALTED        = 0x0000_0040;
        const ACTIVE        = 0x0000_0080;
        const DATA_TOGGLE   = 0x8000_0000;
    }
}

pub const PID_OUT: u32 = 0;
pub const PID_IN: u32 = 1;
pub const PID_SETUP: u32 = 2;

#[repr(C, align(32))]
pub struct TransferDescriptor {
  pub next_qtd: u32,
  pub alt_next_qtd: u32,
  pub token: u32,
  pub buffer_ptrs: [u32; 5],
}

impl TransferDescriptor {
  pub const fn empty() -> Self {
    Self {
      next_qtd: 1,
      alt_next_qtd: 1,
      token: 0,
      buffer_ptrs: [0; 5],
    }
  }

  pub fn set_active(&mut self) {
    self.token |= QtdToken::ACTIVE.bits();
  }

  pub fn set_pid(&mut self, pid: u32) {
    self.token = (self.token & !(0x03 << 8)) | ((pid & 0x03) << 8);
  }

  pub fn set_total_bytes(&mut self, bytes: u16) {
    self.token = (self.token & !0x7FFF_0000) | ((bytes as u32 & 0x7FFF) << 16);
  }

  pub fn set_data_toggle(&mut self, toggle: bool) {
    if toggle {
      self.token |= QtdToken::DATA_TOGGLE.bits();
    } else {
      self.token &= !QtdToken::DATA_TOGGLE.bits();
    }
  }

  pub fn set_buffer(&mut self, index: usize, addr: u32) {
    if index < 5 {
      self.buffer_ptrs[index] = addr;
    }
  }

  pub fn is_active(&self) -> bool {
    self.token & QtdToken::ACTIVE.bits() != 0
  }

  pub fn is_halted(&self) -> bool {
    self.token & QtdToken::HALTED.bits() != 0
  }

  pub fn bytes_transferred(&self, total: u16) -> u16 {
    let remaining = ((self.token >> 16) & 0x7FFF) as u16;
    total.saturating_sub(remaining)
  }
}
