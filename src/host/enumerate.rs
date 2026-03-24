use crate::usb::consts;
use crate::usb::ehci::EhciHost;
use crate::usb::transaction::TransferError;
use heapless::Vec;

#[derive(Debug, Clone, defmt::Format)]
pub struct EndpointDescriptor {
  pub address: u8,
  pub attributes: u8,
  pub max_packet_size: u16,
  pub interval: u8,
}

impl EndpointDescriptor {
  pub fn is_interrupt_in(&self) -> bool {
    (self.attributes & 0x03) == consts::EP_TYPE_INTERRUPT && (self.address & 0x80) != 0
  }

  pub fn is_interrupt_out(&self) -> bool {
    (self.attributes & 0x03) == consts::EP_TYPE_INTERRUPT && (self.address & 0x80) == 0
  }

  pub fn is_bulk_in(&self) -> bool {
    (self.attributes & 0x03) == consts::EP_TYPE_BULK && (self.address & 0x80) != 0
  }

  pub fn is_bulk_out(&self) -> bool {
    (self.attributes & 0x03) == consts::EP_TYPE_BULK && (self.address & 0x80) == 0
  }

  pub fn number(&self) -> u8 {
    self.address & 0x0F
  }

  pub fn direction_in(&self) -> bool {
    (self.address & 0x80) != 0
  }

  pub fn transfer_type(&self) -> u8 {
    self.attributes & 0x03
  }
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
  pub number: u8,
  pub alt_setting: u8,
  pub class: u8,
  pub subclass: u8,
  pub protocol: u8,
  pub endpoints: Vec<EndpointDescriptor, 8>,
  pub hid_report_desc_len: Option<u16>,
}

impl InterfaceInfo {
  pub fn is_hid_mouse(&self) -> bool {
    self.class == consts::CLASS_HID && self.subclass == consts::SUBCLASS_BOOT && self.protocol == consts::PROTOCOL_MOUSE
  }

  pub fn is_hid(&self) -> bool {
    self.class == consts::CLASS_HID
  }

  pub fn mouse_interrupt_in(&self) -> Option<&EndpointDescriptor> {
    if self.is_hid_mouse() {
      self.endpoints.iter().find(|ep| ep.is_interrupt_in())
    } else {
      None
    }
  }
}

pub struct EnumeratedDevice {
  pub address: u8,
  pub raw_device_desc: [u8; 18],
  pub raw_config_desc: Vec<u8, 512>,
  pub interfaces: Vec<InterfaceInfo, 8>,
  pub hid_report_descs: Vec<HidReportDescEntry, 4>,
  pub string_descs: Vec<StringDescEntry, 16>,
}

pub struct HidReportDescEntry {
  pub interface: u8,
  pub data: Vec<u8, 512>,
}

pub struct StringDescEntry {
  pub index: u8,
  pub data: Vec<u8, 128>,
}

impl EnumeratedDevice {
  pub fn mouse_interface(&self) -> Option<&InterfaceInfo> {
    self.interfaces.iter().find(|i| i.is_hid_mouse())
  }

  pub fn hid_report_desc_for(&self, interface: u8) -> Option<&[u8]> {
    self
      .hid_report_descs
      .iter()
      .find(|e| e.interface == interface)
      .map(|e| e.data.as_slice())
  }

  pub fn string_desc(&self, index: u8) -> Option<&[u8]> {
    self
      .string_descs
      .iter()
      .find(|e| e.index == index)
      .map(|e| e.data.as_slice())
  }

  pub fn string_indices_referenced(&self) -> Vec<u8, 16> {
    let mut indices = Vec::new();
    let d = &self.raw_device_desc;

    // device descriptor: iManufacturer(14), iProduct(15), iSerialNumber(16)
    for &idx in &[d[14], d[15], d[16]] {
      if idx != 0 && !indices.contains(&idx) {
        let _ = indices.push(idx);
      }
    }

    // walk config descriptor for iConfiguration and iInterface strings
    let mut offset = 0;
    let cfg = self.raw_config_desc.as_slice();
    while offset + 1 < cfg.len() {
      let len = cfg[offset] as usize;
      let desc_type = cfg[offset + 1];
      if len < 2 || offset + len > cfg.len() {
        break;
      }
      match desc_type {
        consts::DESC_CONFIGURATION if len >= 7 => {
          let idx = cfg[offset + 6];
          if idx != 0 && !indices.contains(&idx) {
            let _ = indices.push(idx);
          }
        }
        consts::DESC_INTERFACE if len >= 9 => {
          let idx = cfg[offset + 8];
          if idx != 0 && !indices.contains(&idx) {
            let _ = indices.push(idx);
          }
        }
        _ => {}
      }
      offset += len;
    }

    indices
  }

  pub fn config_value(&self) -> u8 {
    if self.raw_config_desc.len() >= 6 {
      self.raw_config_desc[5]
    } else {
      1
    }
  }
}

pub fn enumerate_device<const N: u8>(host: &mut EhciHost<N>) -> Result<EnumeratedDevice, TransferError> {
  let _ = host;
  todo!()
}

// SETUP packet builders return [u8; 8] — see usb::consts module for constructors.
// The enumeration sequence is:
//
// 1. GET_DESCRIPTOR(Device) at addr 0, 8 bytes → learn bMaxPacketSize0
// 2. Port reset again
// 3. SET_ADDRESS(1)
// 4. GET_DESCRIPTOR(Device) at addr 1, 18 bytes → store raw
// 5. GET_DESCRIPTOR(Config) at addr 1, 9 bytes → learn wTotalLength
// 6. GET_DESCRIPTOR(Config) at addr 1, wTotalLength → store raw, parse interfaces
// 7. GET_DESCRIPTOR(String) for each referenced index → store raw
// 8. SET_CONFIGURATION
// 9. GET_DESCRIPTOR(HID Report) for each HID interface → store raw
// 10. SET_IDLE for each HID interface (ignore NAK)

pub fn parse_config_descriptor(raw: &[u8]) -> Vec<InterfaceInfo, 8> {
  let mut interfaces = Vec::new();
  let mut offset = 0;
  let mut current_interface: Option<InterfaceInfo> = None;

  while offset + 1 < raw.len() {
    let len = raw[offset] as usize;
    let desc_type = raw[offset + 1];

    if len < 2 || offset + len > raw.len() {
      break;
    }

    match desc_type {
      consts::DESC_INTERFACE if len >= 9 => {
        // Push previous interface if any
        if let Some(iface) = current_interface.take() {
          let _ = interfaces.push(iface);
        }
        current_interface = Some(InterfaceInfo {
          number: raw[offset + 2],
          alt_setting: raw[offset + 3],
          class: raw[offset + 5],
          subclass: raw[offset + 6],
          protocol: raw[offset + 7],
          endpoints: Vec::new(),
          hid_report_desc_len: None,
        });
      }
      consts::DESC_HID if len >= 9 => {
        // HID class descriptor — extract wDescriptorLength at offset 7-8
        if let Some(ref mut iface) = current_interface {
          let report_len = raw[offset + 7] as u16 | ((raw[offset + 8] as u16) << 8);
          iface.hid_report_desc_len = Some(report_len);
        }
      }
      consts::DESC_ENDPOINT if len >= 7 => {
        if let Some(ref mut iface) = current_interface {
          let ep = EndpointDescriptor {
            address: raw[offset + 2],
            attributes: raw[offset + 3],
            max_packet_size: raw[offset + 4] as u16 | ((raw[offset + 5] as u16) << 8),
            interval: raw[offset + 6],
          };
          let _ = iface.endpoints.push(ep);
        }
      }
      _ => {}
    }

    offset += len;
  }

  // Push last interface
  if let Some(iface) = current_interface {
    let _ = interfaces.push(iface);
  }

  interfaces
}
