use imxrt_ral as ral;

pub struct EhciHost<const N: u8> {
  _usb: ral::usb::Instance<N>,
}

impl<const N: u8> EhciHost<N> {
  pub fn new(usb: ral::usb::Instance<N>) -> Self {
    Self { _usb: usb }
  }

  pub fn reset(&mut self) {
    todo!()
  }

  pub fn detect_device(&self) -> bool {
    todo!()
  }

  pub fn port_reset(&mut self) {
    todo!()
  }

  pub fn enable_periodic_schedule(&mut self) {
    todo!()
  }

  pub fn enable_async_schedule(&mut self) {
    todo!()
  }
}
