#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use defmt_rtt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
  let bsp::board::Resources { mut gpio2, pins, .. } = bsp::board::t41(bsp::board::instances());

  let led = bsp::board::led(&mut gpio2, pins.p13);

  defmt::info!("blink starting");

  loop {
    led.toggle();
    cortex_m::asm::delay(150_000_000);
  }
}
