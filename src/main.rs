#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_stm32::init;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = init(Default::default());

    info!("Hello from Embassy STM32G0!");
}
