pub struct InterruptInEndpoint {
  _endpoint_num: u8,
}

impl InterruptInEndpoint {
  pub fn new(endpoint_num: u8) -> Self {
    Self {
      _endpoint_num: endpoint_num,
    }
  }

  pub fn send_report(&mut self, _report: &[u8]) {
    todo!()
  }

  pub fn is_ready(&self) -> bool {
    todo!()
  }
}
