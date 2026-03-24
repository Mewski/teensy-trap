// USB descriptor types
pub const DESC_DEVICE: u8 = 0x01;
pub const DESC_CONFIGURATION: u8 = 0x02;
pub const DESC_STRING: u8 = 0x03;
pub const DESC_INTERFACE: u8 = 0x04;
pub const DESC_ENDPOINT: u8 = 0x05;
pub const DESC_HID: u8 = 0x21;
pub const DESC_HID_REPORT: u8 = 0x22;

// USB standard requests
pub const REQ_GET_STATUS: u8 = 0x00;
pub const REQ_CLEAR_FEATURE: u8 = 0x01;
pub const REQ_SET_FEATURE: u8 = 0x03;
pub const REQ_SET_ADDRESS: u8 = 0x05;
pub const REQ_GET_DESCRIPTOR: u8 = 0x06;
pub const REQ_SET_CONFIGURATION: u8 = 0x09;
pub const REQ_SET_INTERFACE: u8 = 0x0B;

// HID class requests
pub const REQ_HID_GET_REPORT: u8 = 0x01;
pub const REQ_HID_SET_IDLE: u8 = 0x0A;
pub const REQ_HID_SET_PROTOCOL: u8 = 0x0B;

// bmRequestType
pub const RT_DEV_TO_HOST: u8 = 0x80;
pub const RT_HOST_TO_DEV: u8 = 0x00;
pub const RT_STANDARD: u8 = 0x00;
pub const RT_CLASS: u8 = 0x20;
pub const RT_VENDOR: u8 = 0x40;
pub const RT_RECIPIENT_DEVICE: u8 = 0x00;
pub const RT_RECIPIENT_INTERFACE: u8 = 0x01;
pub const RT_RECIPIENT_ENDPOINT: u8 = 0x02;

// Endpoint transfer types (bmAttributes bits 1:0)
pub const EP_TYPE_CONTROL: u8 = 0x00;
pub const EP_TYPE_ISOCHRONOUS: u8 = 0x01;
pub const EP_TYPE_BULK: u8 = 0x02;
pub const EP_TYPE_INTERRUPT: u8 = 0x03;

// HID interface class/subclass/protocol
pub const CLASS_HID: u8 = 0x03;
pub const SUBCLASS_BOOT: u8 = 0x01;
pub const PROTOCOL_MOUSE: u8 = 0x02;

// USB language ID
pub const LANGID_EN_US: u16 = 0x0409;

pub fn setup_get_descriptor(desc_type: u8, index: u8, length: u16) -> [u8; 8] {
  [
    RT_DEV_TO_HOST | RT_STANDARD | RT_RECIPIENT_DEVICE,
    REQ_GET_DESCRIPTOR,
    index,
    desc_type,
    0x00,
    0x00,
    length as u8,
    (length >> 8) as u8,
  ]
}

pub fn setup_get_string_descriptor(index: u8, lang_id: u16, length: u16) -> [u8; 8] {
  [
    RT_DEV_TO_HOST | RT_STANDARD | RT_RECIPIENT_DEVICE,
    REQ_GET_DESCRIPTOR,
    index,
    DESC_STRING,
    lang_id as u8,
    (lang_id >> 8) as u8,
    length as u8,
    (length >> 8) as u8,
  ]
}

pub fn setup_get_hid_report_descriptor(interface: u8, length: u16) -> [u8; 8] {
  [
    RT_DEV_TO_HOST | RT_STANDARD | RT_RECIPIENT_INTERFACE,
    REQ_GET_DESCRIPTOR,
    0x00,
    DESC_HID_REPORT,
    interface,
    0x00,
    length as u8,
    (length >> 8) as u8,
  ]
}

pub fn setup_set_address(addr: u8) -> [u8; 8] {
  [
    RT_HOST_TO_DEV | RT_STANDARD | RT_RECIPIENT_DEVICE,
    REQ_SET_ADDRESS,
    addr,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
  ]
}

pub fn setup_set_configuration(config: u8) -> [u8; 8] {
  [
    RT_HOST_TO_DEV | RT_STANDARD | RT_RECIPIENT_DEVICE,
    REQ_SET_CONFIGURATION,
    config,
    0x00,
    0x00,
    0x00,
    0x00,
    0x00,
  ]
}

pub fn setup_set_idle(interface: u8) -> [u8; 8] {
  [
    RT_HOST_TO_DEV | RT_CLASS | RT_RECIPIENT_INTERFACE,
    REQ_HID_SET_IDLE,
    0x00,
    0x00,
    interface,
    0x00,
    0x00,
    0x00,
  ]
}
