use super::qh::QueueHead;
use super::qtd::TransferDescriptor;

pub struct ControlTransfer<'a> {
  pub qh: &'a mut QueueHead,
  pub setup_qtd: &'a mut TransferDescriptor,
  pub data_qtd: &'a mut TransferDescriptor,
  pub status_qtd: &'a mut TransferDescriptor,
}

impl<'a> ControlTransfer<'a> {
  pub fn prepare_setup(&mut self, setup_packet: &[u8; 8], setup_buf: &mut [u8; 8]) {
    setup_buf.copy_from_slice(setup_packet);
    self.setup_qtd.set_buffer(0, setup_buf.as_ptr() as u32);
    self.setup_qtd.set_total_bytes(8);
    self.setup_qtd.set_pid(super::qtd::PID_SETUP);
    self.setup_qtd.set_data_toggle(false);
    self.setup_qtd.set_active();
  }

  pub fn prepare_data_in(&mut self, buf: &mut [u8], len: u16) {
    self.data_qtd.set_buffer(0, buf.as_ptr() as u32);
    self.data_qtd.set_total_bytes(len);
    self.data_qtd.set_pid(super::qtd::PID_IN);
    self.data_qtd.set_data_toggle(true);
    self.data_qtd.set_active();
  }

  pub fn prepare_status_out(&mut self) {
    self.status_qtd.set_total_bytes(0);
    self.status_qtd.set_pid(super::qtd::PID_OUT);
    self.status_qtd.set_data_toggle(true);
    self.status_qtd.set_active();
  }
}
