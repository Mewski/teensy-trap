pub struct ProxyEndpoint {
  pub address: u8,
  pub max_packet_size: u16,
}

impl ProxyEndpoint {
  pub fn new(address: u8, max_packet_size: u16) -> Self {
    Self {
      address,
      max_packet_size,
    }
  }

  pub fn send(&mut self, _data: &[u8]) {
    todo!()
  }

  pub fn receive(&mut self, _buf: &mut [u8]) -> Option<usize> {
    todo!()
  }

  pub fn is_ready(&self) -> bool {
    todo!()
  }

  pub fn number(&self) -> u8 {
    self.address & 0x0F
  }

  pub fn direction_in(&self) -> bool {
    (self.address & 0x80) != 0
  }
}
