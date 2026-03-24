use super::ehci::EhciHost;
use super::qtd::TransferDescriptor;

#[derive(Debug, defmt::Format)]
pub enum TransferError {
  Timeout,
  Stall,
  DataBufferError,
  BabbleDetected,
  TransactionError,
  NotReady,
}

pub fn wait_for_qtd(qtd: &TransferDescriptor, timeout_cycles: u32) -> Result<(), TransferError> {
  let mut countdown = timeout_cycles;
  while qtd.is_active() {
    if countdown == 0 {
      return Err(TransferError::Timeout);
    }
    countdown -= 1;
    cortex_m::asm::nop();
  }
  if qtd.is_halted() {
    let token = qtd.token;
    if token & (1 << 5) != 0 {
      return Err(TransferError::DataBufferError);
    }
    if token & (1 << 4) != 0 {
      return Err(TransferError::BabbleDetected);
    }
    if token & (1 << 3) != 0 {
      return Err(TransferError::TransactionError);
    }
    return Err(TransferError::Stall);
  }
  Ok(())
}

pub fn wait_for_transfer<const N: u8>(host: &mut EhciHost<N>, timeout_cycles: u32) -> Result<(), TransferError> {
  let mut countdown = timeout_cycles;
  loop {
    if host.poll_transfer_complete() {
      host.ack_transfer_complete();
      return Ok(());
    }
    if host.poll_error() {
      host.ack_error();
      return Err(TransferError::TransactionError);
    }
    if countdown == 0 {
      return Err(TransferError::Timeout);
    }
    countdown -= 1;
    cortex_m::asm::nop();
  }
}
