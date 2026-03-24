use crate::host::mouse::MouseReport;

pub struct Forwarder {
  mouse_report_count: u64,
  passthrough_count: u64,
  drop_count: u64,
  mouse_endpoint_addr: u8,
}

impl Forwarder {
  pub fn new(mouse_endpoint_addr: u8) -> Self {
    Self {
      mouse_report_count: 0,
      passthrough_count: 0,
      drop_count: 0,
      mouse_endpoint_addr,
    }
  }

  pub fn is_mouse_endpoint(&self, endpoint_addr: u8) -> bool {
    endpoint_addr == self.mouse_endpoint_addr
  }

  pub fn forward_mouse(&mut self, report: &MouseReport, output: &mut [u8]) -> usize {
    self.mouse_report_count += 1;
    let _ = (report, output);
    todo!()
  }

  pub fn forward_passthrough(&mut self, _endpoint_addr: u8, data: &[u8], output: &mut [u8]) -> usize {
    self.passthrough_count += 1;
    let len = data.len().min(output.len());
    output[..len].copy_from_slice(&data[..len]);
    len
  }

  pub fn stats(&self) -> (u64, u64, u64) {
    (self.mouse_report_count, self.passthrough_count, self.drop_count)
  }
}
