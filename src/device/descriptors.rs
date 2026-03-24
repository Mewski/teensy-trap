use crate::host::enumerate::EnumeratedDevice;
use heapless::Vec;

pub struct ClonedDescriptors {
  pub device_desc: [u8; 18],
  pub config_desc: Vec<u8, 512>,
  pub hid_report_descs: Vec<HidReportDescClone, 4>,
  pub string_descs: Vec<StringDescClone, 16>,
}

pub struct HidReportDescClone {
  pub interface: u8,
  pub data: Vec<u8, 512>,
}

pub struct StringDescClone {
  pub index: u8,
  pub data: Vec<u8, 128>,
}

impl ClonedDescriptors {
  pub fn from_upstream(upstream: &EnumeratedDevice) -> Self {
    let mut hid_report_descs = Vec::new();
    for entry in &upstream.hid_report_descs {
      let _ = hid_report_descs.push(HidReportDescClone {
        interface: entry.interface,
        data: entry.data.clone(),
      });
    }

    let mut string_descs = Vec::new();
    for entry in &upstream.string_descs {
      let _ = string_descs.push(StringDescClone {
        index: entry.index,
        data: entry.data.clone(),
      });
    }

    Self {
      device_desc: upstream.raw_device_desc,
      config_desc: upstream.raw_config_desc.clone(),
      hid_report_descs,
      string_descs,
    }
  }

  pub fn string_desc(&self, index: u8) -> Option<&[u8]> {
    self
      .string_descs
      .iter()
      .find(|e| e.index == index)
      .map(|e| e.data.as_slice())
  }

  pub fn hid_report_desc(&self, interface: u8) -> Option<&[u8]> {
    self
      .hid_report_descs
      .iter()
      .find(|e| e.interface == interface)
      .map(|e| e.data.as_slice())
  }
}
