use crate::host::enumerate::EnumeratedDevice;
use heapless::Vec;

pub struct ClonedDescriptors {
  pub device_desc: [u8; 18],
  pub config_desc: Vec<u8, 512>,
  pub hid_report_desc: Vec<u8, 512>,
  pub str_manufacturer: Vec<u8, 128>,
  pub str_product: Vec<u8, 128>,
  pub str_serial_number: Vec<u8, 128>,
  pub interrupt_in_max_packet: u16,
  pub interrupt_in_interval: u8,
}

impl ClonedDescriptors {
  pub fn from_upstream(upstream: &EnumeratedDevice) -> Self {
    Self {
      device_desc: upstream.raw_device_desc,
      config_desc: upstream.raw_config_desc.clone(),
      hid_report_desc: upstream.raw_hid_report_desc.clone(),
      str_manufacturer: upstream.string_descs.manufacturer.clone(),
      str_product: upstream.string_descs.product.clone(),
      str_serial_number: upstream.string_descs.serial_number.clone(),
      interrupt_in_max_packet: upstream.interrupt_in.max_packet_size,
      interrupt_in_interval: upstream.interrupt_in.interval,
    }
  }
}
