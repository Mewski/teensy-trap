#![no_std]
#![no_main]

mod device;
mod host;
mod proxy;
mod serial;
mod usb;

use teensy4_bsp as bsp;
use teensy4_panic as _;

use defmt_rtt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
  let bsp::board::Resources { mut gpio2, pins, .. } = bsp::board::t41(bsp::board::instances());

  let led = bsp::board::led(&mut gpio2, pins.p13);
  led.set();

  defmt::info!("teensytrap starting");

  loop {
    cortex_m::asm::wfi();
  }
}
