use super::descriptors::ClonedDescriptors;
use super::endpoint::InterruptInEndpoint;

pub struct HidMouseDevice {
  _descriptors: ClonedDescriptors,
  _endpoint: InterruptInEndpoint,
}

impl HidMouseDevice {
  pub fn new(descriptors: ClonedDescriptors) -> Self {
    let _ = &descriptors;
    todo!()
  }

  pub fn send_report(&mut self, report: &[u8]) {
    let _ = report;
    todo!()
  }

  pub fn is_configured(&self) -> bool {
    todo!()
  }
}
