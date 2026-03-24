#[derive(Debug, Clone, defmt::Format)]
pub struct MouseFieldLayout {
  pub report_id: Option<u8>,
  pub report_size: u16,
  pub button_offset: u16,
  pub button_count: u8,
  pub x_offset: u16,
  pub x_bits: u8,
  pub x_signed: bool,
  pub y_offset: u16,
  pub y_bits: u8,
  pub y_signed: bool,
  pub wheel_offset: Option<u16>,
  pub wheel_bits: u8,
  pub wheel_signed: bool,
}

impl MouseFieldLayout {
  pub fn parse(descriptor: &[u8]) -> Option<Self> {
    let _ = descriptor;
    todo!()
  }

  pub fn extract_x(&self, report: &[u8]) -> i16 {
    let _ = report;
    todo!()
  }

  pub fn extract_y(&self, report: &[u8]) -> i16 {
    let _ = report;
    todo!()
  }

  pub fn extract_buttons(&self, report: &[u8]) -> u8 {
    let _ = report;
    todo!()
  }

  pub fn extract_wheel(&self, report: &[u8]) -> i8 {
    let _ = report;
    todo!()
  }
}
