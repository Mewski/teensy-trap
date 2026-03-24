use super::ehci::EhciHost;

#[derive(Debug, defmt::Format)]
pub enum TransferError {
  Timeout,
  Stall,
  DataBufferError,
  BabbleDetected,
  TransactionError,
  NotReady,
}

pub fn poll_transfer_complete<const N: u8>(host: &EhciHost<N>) -> Result<(), TransferError> {
  let _ = host;
  todo!()
}
