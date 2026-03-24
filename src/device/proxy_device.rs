use super::descriptors::ClonedDescriptors;
use super::endpoint::ProxyEndpoint;
use heapless::Vec;

pub struct ProxyDevice {
  descriptors: ClonedDescriptors,
  endpoints: Vec<ProxyEndpoint, 14>,
  mouse_endpoint_addr: u8,
}

impl ProxyDevice {
  pub fn new(descriptors: ClonedDescriptors, mouse_endpoint_addr: u8) -> Self {
    let _ = (&descriptors, mouse_endpoint_addr);
    todo!()
  }

  pub fn send_to_endpoint(&mut self, endpoint_addr: u8, data: &[u8]) {
    let _ = (endpoint_addr, data);
    todo!()
  }

  pub fn receive_from_endpoint(&mut self, endpoint_addr: u8, buf: &mut [u8]) -> Option<usize> {
    let _ = (endpoint_addr, buf);
    todo!()
  }

  pub fn forward_control_to_upstream(&mut self, _setup: &[u8; 8], _data: &[u8]) -> Option<Vec<u8, 256>> {
    todo!()
  }

  pub fn is_configured(&self) -> bool {
    todo!()
  }

  pub fn mouse_endpoint_addr(&self) -> u8 {
    self.mouse_endpoint_addr
  }

  pub fn descriptors(&self) -> &ClonedDescriptors {
    &self.descriptors
  }

  pub fn endpoints(&self) -> &[ProxyEndpoint] {
    &self.endpoints
  }
}
