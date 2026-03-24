use super::qh::QueueHead;

const FRAME_LIST_SIZE: usize = 1024;

#[repr(C, align(4096))]
pub struct PeriodicFrameList {
  entries: [u32; FRAME_LIST_SIZE],
}

impl PeriodicFrameList {
  pub const fn empty() -> Self {
    Self {
      entries: [1; FRAME_LIST_SIZE],
    }
  }

  pub fn base_addr(&self) -> u32 {
    self.entries.as_ptr() as u32
  }

  pub fn link_interrupt_qh(&mut self, qh: &QueueHead) {
    let qh_addr = (qh as *const QueueHead) as u32;
    let link = (qh_addr & !0x1F) | 0x02;
    for entry in &mut self.entries {
      *entry = link;
    }
  }

  pub fn clear(&mut self) {
    for entry in &mut self.entries {
      *entry = 1;
    }
  }
}
