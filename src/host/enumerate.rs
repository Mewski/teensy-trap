use crate::usb::ehci::EhciHost;
use crate::usb::transaction::TransferError;
use heapless::Vec;

#[derive(Debug, defmt::Format)]
pub struct InterruptEndpoint {
  pub address: u8,
  pub max_packet_size: u16,
  pub interval: u8,
}

pub struct EnumeratedDevice {
  pub address: u8,
  pub raw_device_desc: [u8; 18],
  pub raw_config_desc: Vec<u8, 512>,
  pub raw_hid_report_desc: Vec<u8, 512>,
  pub string_descs: StringDescriptorTable,
  pub interrupt_in: InterruptEndpoint,
}

pub struct StringDescriptorTable {
  pub manufacturer: Vec<u8, 128>,
  pub product: Vec<u8, 128>,
  pub serial_number: Vec<u8, 128>,
}

pub fn enumerate_mouse<const N: u8>(host: &mut EhciHost<N>) -> Result<EnumeratedDevice, TransferError> {
  let _ = host;
  todo!()
}

fn set_address<const N: u8>(_host: &mut EhciHost<N>, _addr: u8) -> Result<(), TransferError> {
  todo!()
}

fn get_device_descriptor<const N: u8>(
  _host: &mut EhciHost<N>,
  _addr: u8,
  _buf: &mut [u8; 18],
) -> Result<(), TransferError> {
  todo!()
}

fn get_config_descriptor<const N: u8>(
  _host: &mut EhciHost<N>,
  _addr: u8,
  _buf: &mut [u8],
) -> Result<usize, TransferError> {
  todo!()
}

fn get_string_descriptor<const N: u8>(
  _host: &mut EhciHost<N>,
  _addr: u8,
  _index: u8,
  _buf: &mut [u8],
) -> Result<usize, TransferError> {
  todo!()
}

fn get_hid_report_descriptor<const N: u8>(
  _host: &mut EhciHost<N>,
  _addr: u8,
  _interface: u8,
  _len: u16,
  _buf: &mut [u8],
) -> Result<usize, TransferError> {
  todo!()
}

fn set_configuration<const N: u8>(_host: &mut EhciHost<N>, _addr: u8, _config: u8) -> Result<(), TransferError> {
  todo!()
}
