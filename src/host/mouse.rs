use super::hid::MouseFieldLayout;

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct MouseReport {
  pub buttons: u8,
  pub x: i16,
  pub y: i16,
  pub wheel: i8,
}

pub struct MouseReader {
  layout: MouseFieldLayout,
}

impl MouseReader {
  pub fn new(layout: MouseFieldLayout) -> Self {
    Self { layout }
  }

  pub fn parse_report(&self, raw: &[u8]) -> MouseReport {
    MouseReport {
      buttons: self.layout.extract_buttons(raw),
      x: self.layout.extract_x(raw),
      y: self.layout.extract_y(raw),
      wheel: self.layout.extract_wheel(raw),
    }
  }

  pub fn layout(&self) -> &MouseFieldLayout {
    &self.layout
  }
}
