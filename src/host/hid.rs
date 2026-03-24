#[derive(Debug, Clone, defmt::Format)]
pub struct MouseFieldLayout {
  pub report_id: Option<u8>,
  pub report_byte_len: u16,
  pub button_bit_offset: u16,
  pub button_count: u8,
  pub x_bit_offset: u16,
  pub x_bits: u8,
  pub x_signed: bool,
  pub y_bit_offset: u16,
  pub y_bits: u8,
  pub y_signed: bool,
  pub wheel_bit_offset: Option<u16>,
  pub wheel_bits: u8,
  pub wheel_signed: bool,
}

impl MouseFieldLayout {
  pub fn parse(descriptor: &[u8]) -> Option<Self> {
    let mut parser = HidParser::new(descriptor);
    parser.find_mouse_fields()
  }

  pub fn extract_buttons(&self, report: &[u8]) -> u8 {
    let offset = self.byte_start(report);
    let mut buttons = 0u8;
    for i in 0..self.button_count.min(8) {
      let bit = self.button_bit_offset + i as u16;
      if get_bit(offset, bit) {
        buttons |= 1 << i;
      }
    }
    buttons
  }

  pub fn extract_x(&self, report: &[u8]) -> i16 {
    let offset = self.byte_start(report);
    extract_field(offset, self.x_bit_offset, self.x_bits, self.x_signed)
  }

  pub fn extract_y(&self, report: &[u8]) -> i16 {
    let offset = self.byte_start(report);
    extract_field(offset, self.y_bit_offset, self.y_bits, self.y_signed)
  }

  pub fn extract_wheel(&self, report: &[u8]) -> i8 {
    match self.wheel_bit_offset {
      Some(offset) => {
        let data = self.byte_start(report);
        extract_field(data, offset, self.wheel_bits, self.wheel_signed) as i8
      }
      None => 0,
    }
  }

  fn byte_start<'a>(&self, report: &'a [u8]) -> &'a [u8] {
    if self.report_id.is_some() {
      // Skip the report ID prefix byte
      &report[1..]
    } else {
      report
    }
  }
}

fn get_bit(data: &[u8], bit_offset: u16) -> bool {
  let byte_idx = (bit_offset / 8) as usize;
  let bit_idx = bit_offset % 8;
  if byte_idx < data.len() {
    data[byte_idx] & (1 << bit_idx) != 0
  } else {
    false
  }
}

fn extract_field(data: &[u8], bit_offset: u16, bits: u8, signed: bool) -> i16 {
  let mut val: u32 = 0;
  for i in 0..bits as u16 {
    if get_bit(data, bit_offset + i) {
      val |= 1 << i;
    }
  }
  if signed && bits > 0 && val & (1 << (bits - 1)) != 0 {
    // Sign extend
    val |= !((1u32 << bits) - 1);
  }
  val as i16
}

// HID item types
const ITEM_TYPE_MAIN: u8 = 0;
const ITEM_TYPE_GLOBAL: u8 = 1;
const ITEM_TYPE_LOCAL: u8 = 2;

// Main item tags
const TAG_INPUT: u8 = 0x8;
const TAG_COLLECTION: u8 = 0xA;

// Global item tags
const TAG_USAGE_PAGE: u8 = 0x0;
const TAG_LOGICAL_MIN: u8 = 0x1;
const TAG_LOGICAL_MAX: u8 = 0x2;
const TAG_REPORT_SIZE: u8 = 0x7;
const TAG_REPORT_ID: u8 = 0x8;
const TAG_REPORT_COUNT: u8 = 0x9;

// Local item tags
const TAG_USAGE: u8 = 0x0;
const TAG_USAGE_MIN: u8 = 0x1;
const TAG_USAGE_MAX: u8 = 0x2;

// Usage pages
const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
const USAGE_PAGE_BUTTON: u16 = 0x09;

// Usages (Generic Desktop page)
const USAGE_MOUSE: u8 = 0x02;
const USAGE_X: u8 = 0x30;
const USAGE_Y: u8 = 0x31;
const USAGE_WHEEL: u8 = 0x38;

struct HidParser<'a> {
  data: &'a [u8],
  pos: usize,
  // Global state
  usage_page: u16,
  logical_min: i32,
  logical_max: i32,
  report_size: u8,
  report_count: u8,
  report_id: Option<u8>,
  // Local state
  usages: [u8; 16],
  usage_count: usize,
  usage_min: u8,
  usage_max: u8,
  // Tracking
  bit_offset: u16,
  found_mouse_collection: bool,
  // Result
  button_bit_offset: Option<u16>,
  button_count: Option<u8>,
  x_bit_offset: Option<u16>,
  x_bits: Option<u8>,
  x_signed: bool,
  y_bit_offset: Option<u16>,
  y_bits: Option<u8>,
  y_signed: bool,
  wheel_bit_offset: Option<u16>,
  wheel_bits: Option<u8>,
  wheel_signed: bool,
}

impl<'a> HidParser<'a> {
  fn new(data: &'a [u8]) -> Self {
    Self {
      data,
      pos: 0,
      usage_page: 0,
      logical_min: 0,
      logical_max: 0,
      report_size: 0,
      report_count: 0,
      report_id: None,
      usages: [0; 16],
      usage_count: 0,
      usage_min: 0,
      usage_max: 0,
      bit_offset: 0,
      found_mouse_collection: false,
      button_bit_offset: None,
      button_count: None,
      x_bit_offset: None,
      x_bits: None,
      x_signed: false,
      y_bit_offset: None,
      y_bits: None,
      y_signed: false,
      wheel_bit_offset: None,
      wheel_bits: None,
      wheel_signed: false,
    }
  }

  fn find_mouse_fields(&mut self) -> Option<MouseFieldLayout> {
    while self.pos < self.data.len() {
      let prefix = self.data[self.pos];
      self.pos += 1;

      if prefix == 0xFE {
        // Long item — skip
        if self.pos + 2 > self.data.len() {
          break;
        }
        let size = self.data[self.pos] as usize;
        self.pos += 2 + size;
        continue;
      }

      let bsize = prefix & 0x03;
      let btype = (prefix >> 2) & 0x03;
      let btag = (prefix >> 4) & 0x0F;
      let data_len = if bsize == 3 { 4usize } else { bsize as usize };

      if self.pos + data_len > self.data.len() {
        break;
      }

      let data_val = self.read_item_data(data_len);
      self.pos += data_len;

      match btype {
        ITEM_TYPE_GLOBAL => self.handle_global(btag, data_val),
        ITEM_TYPE_LOCAL => self.handle_local(btag, data_val),
        ITEM_TYPE_MAIN => self.handle_main(btag, data_val),
        _ => {}
      }
    }

    let is_signed = self.logical_min < 0;
    let total_bits = self.bit_offset;

    if let (Some(btn_off), Some(x_off), Some(y_off)) = (self.button_bit_offset, self.x_bit_offset, self.y_bit_offset) {
      let report_byte_len = total_bits.div_ceil(8);
      Some(MouseFieldLayout {
        report_id: self.report_id,
        report_byte_len,
        button_bit_offset: btn_off,
        button_count: self.button_count.unwrap_or(3),
        x_bit_offset: x_off,
        x_bits: self.x_bits.unwrap_or(8),
        x_signed: self.x_signed || is_signed,
        y_bit_offset: y_off,
        y_bits: self.y_bits.unwrap_or(8),
        y_signed: self.y_signed || is_signed,
        wheel_bit_offset: self.wheel_bit_offset,
        wheel_bits: self.wheel_bits.unwrap_or(8),
        wheel_signed: self.wheel_signed || is_signed,
      })
    } else {
      None
    }
  }

  fn read_item_data(&self, len: usize) -> i32 {
    match len {
      0 => 0,
      1 => self.data[self.pos] as i8 as i32,
      2 => {
        let lo = self.data[self.pos] as u16;
        let hi = self.data[self.pos + 1] as u16;
        (lo | (hi << 8)) as i16 as i32
      }
      4 => {
        let b = &self.data[self.pos..];
        i32::from_le_bytes([b[0], b[1], b[2], b[3]])
      }
      _ => 0,
    }
  }

  fn handle_global(&mut self, tag: u8, data: i32) {
    match tag {
      TAG_USAGE_PAGE => self.usage_page = data as u16,
      TAG_LOGICAL_MIN => self.logical_min = data,
      TAG_LOGICAL_MAX => self.logical_max = data,
      TAG_REPORT_SIZE => self.report_size = data as u8,
      TAG_REPORT_COUNT => self.report_count = data as u8,
      TAG_REPORT_ID => self.report_id = Some(data as u8),
      _ => {}
    }
  }

  fn handle_local(&mut self, tag: u8, data: i32) {
    match tag {
      TAG_USAGE => {
        if self.usage_count < self.usages.len() {
          self.usages[self.usage_count] = data as u8;
          self.usage_count += 1;
        }
      }
      TAG_USAGE_MIN => self.usage_min = data as u8,
      TAG_USAGE_MAX => self.usage_max = data as u8,
      _ => {}
    }
  }

  fn handle_main(&mut self, tag: u8, data: i32) {
    if tag == TAG_COLLECTION {
      // Application collection (0x01) with mouse usage
      if data == 0x01
        && self.usage_page == USAGE_PAGE_GENERIC_DESKTOP
        && self.usage_count > 0
        && self.usages[0] == USAGE_MOUSE
      {
        self.found_mouse_collection = true;
      }
      self.clear_local();
      return;
    }

    if tag == TAG_INPUT {
      let is_constant = data & 0x01 != 0;
      let total_field_bits = self.report_size as u16 * self.report_count as u16;

      if !is_constant && self.found_mouse_collection {
        let is_signed = self.logical_min < 0;

        if self.usage_page == USAGE_PAGE_BUTTON {
          self.button_bit_offset = Some(self.bit_offset);
          self.button_count = Some(self.report_count);
        } else if self.usage_page == USAGE_PAGE_GENERIC_DESKTOP {
          // Walk usages to assign axes
          for i in 0..self.usage_count.min(self.report_count as usize) {
            let usage = self.usages[i];
            let field_offset = self.bit_offset + (i as u16 * self.report_size as u16);
            match usage {
              USAGE_X => {
                self.x_bit_offset = Some(field_offset);
                self.x_bits = Some(self.report_size);
                self.x_signed = is_signed;
              }
              USAGE_Y => {
                self.y_bit_offset = Some(field_offset);
                self.y_bits = Some(self.report_size);
                self.y_signed = is_signed;
              }
              USAGE_WHEEL => {
                self.wheel_bit_offset = Some(field_offset);
                self.wheel_bits = Some(self.report_size);
                self.wheel_signed = is_signed;
              }
              _ => {}
            }
          }
          // Handle usage_min/max range (e.g. X=0x30, Y=0x31 via range)
          if self.usage_count == 0 && self.usage_min > 0 {
            for u in self.usage_min..=self.usage_max {
              let i = (u - self.usage_min) as u16;
              let field_offset = self.bit_offset + i * self.report_size as u16;
              match u {
                USAGE_X => {
                  self.x_bit_offset = Some(field_offset);
                  self.x_bits = Some(self.report_size);
                  self.x_signed = is_signed;
                }
                USAGE_Y => {
                  self.y_bit_offset = Some(field_offset);
                  self.y_bits = Some(self.report_size);
                  self.y_signed = is_signed;
                }
                USAGE_WHEEL => {
                  self.wheel_bit_offset = Some(field_offset);
                  self.wheel_bits = Some(self.report_size);
                  self.wheel_signed = is_signed;
                }
                _ => {}
              }
            }
          }
        }
      }

      self.bit_offset += total_field_bits;
      self.clear_local();
    }
  }

  fn clear_local(&mut self) {
    self.usage_count = 0;
    self.usage_min = 0;
    self.usage_max = 0;
    self.usages = [0; 16];
  }
}
