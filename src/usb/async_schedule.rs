use super::qh::QueueHead;
use super::qtd::{self, TransferDescriptor};

pub struct ControlTransfer<'a> {
  pub qh: &'a mut QueueHead,
  pub setup_qtd: &'a mut TransferDescriptor,
  pub data_qtd: &'a mut TransferDescriptor,
  pub status_qtd: &'a mut TransferDescriptor,
}

impl<'a> ControlTransfer<'a> {
  /// Chain qTDs: setup → data → status (or setup → status for no-data transfers).
  /// Must be called after preparing individual qTDs.
  pub fn chain_setup_data_status(&mut self) {
    let data_addr = (self.data_qtd as *const TransferDescriptor) as u32;
    let status_addr = (self.status_qtd as *const TransferDescriptor) as u32;
    self.setup_qtd.next_qtd = data_addr & !0x1F;
    self.data_qtd.next_qtd = status_addr & !0x1F;
    // status_qtd.next_qtd stays as TERMINATE (1)
    self.qh.link_qtd((self.setup_qtd as *const TransferDescriptor) as u32);
  }

  /// Chain qTDs for a no-data control transfer: setup → status.
  pub fn chain_setup_status(&mut self) {
    let status_addr = (self.status_qtd as *const TransferDescriptor) as u32;
    self.setup_qtd.next_qtd = status_addr & !0x1F;
    self.qh.link_qtd((self.setup_qtd as *const TransferDescriptor) as u32);
  }

  // NOTE: All buffers passed to these functions must be in DMA-accessible memory.
  // On iMXRT1062, DTCM (0x2000_0000 - 0x2007_FFFF) is DMA-accessible.
  // Static/global buffers and stack are in DTCM by default on Teensy 4.1.

  pub fn prepare_setup(&mut self, setup_packet: &[u8; 8], buf: &mut [u8; 8]) {
    buf.copy_from_slice(setup_packet);
    *self.setup_qtd = TransferDescriptor::empty();
    self.setup_qtd.set_buffer(0, buf.as_ptr() as u32);
    self.setup_qtd.set_total_bytes(8);
    self.setup_qtd.set_pid(qtd::PID_SETUP);
    self.setup_qtd.set_data_toggle(false);
    self.setup_qtd.set_active();
  }

  pub fn prepare_data_in(&mut self, buf: &mut [u8], len: u16) {
    *self.data_qtd = TransferDescriptor::empty();
    self.data_qtd.set_buffer(0, buf.as_ptr() as u32);
    self.data_qtd.set_total_bytes(len);
    self.data_qtd.set_pid(qtd::PID_IN);
    self.data_qtd.set_data_toggle(true);
    self.data_qtd.set_active();
  }

  pub fn prepare_data_out(&mut self, buf: &[u8], len: u16) {
    *self.data_qtd = TransferDescriptor::empty();
    self.data_qtd.set_buffer(0, buf.as_ptr() as u32);
    self.data_qtd.set_total_bytes(len);
    self.data_qtd.set_pid(qtd::PID_OUT);
    self.data_qtd.set_data_toggle(true);
    self.data_qtd.set_active();
  }

  pub fn prepare_status_out(&mut self) {
    *self.status_qtd = TransferDescriptor::empty();
    self.status_qtd.set_total_bytes(0);
    self.status_qtd.set_pid(qtd::PID_OUT);
    self.status_qtd.set_data_toggle(true);
    self.status_qtd.set_active();
  }

  pub fn prepare_status_in(&mut self) {
    *self.status_qtd = TransferDescriptor::empty();
    self.status_qtd.set_total_bytes(0);
    self.status_qtd.set_pid(qtd::PID_IN);
    self.status_qtd.set_data_toggle(true);
    self.status_qtd.set_active();
  }
}
