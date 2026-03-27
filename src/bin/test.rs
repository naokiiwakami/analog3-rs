#![no_std]
#![no_main]

use defmt::{assert, info};
use defmt_rtt as _;
use defmt_test as _;
use embassy_executor::{Executor, Spawner, task};
use embassy_stm32 as _;
use embassy_stm32::flash::Flash;
use embedded_storage::nor_flash::ReadNorFlash;
use panic_probe as _;

#[embassy_executor::task]
async fn async_task(mut flash: Flash<'static, embassy_stm32::flash::Blocking>) {
    let mut buf = [0u8; 16];

    flash.read(0, &mut buf).unwrap();

    info!("Async flash: {:x}", buf);

    // cortex_m::asm::bkpt(); // end test
}

async fn read_something() -> u32 {
    42
}

#[defmt_test::tests]
mod tests {
    use defmt::{assert, info};
    // use defmt::*;
    use crate::async_task;
    use analog3::storage::Storage;
    use embassy_executor::{Executor, Spawner};
    use embassy_futures::block_on;
    use embassy_stm32::{Peripherals, bind_interrupts, interrupt};
    use embassy_stm32::{
        flash::{self, Flash, InterruptHandler},
        init,
    };
    use embedded_storage::nor_flash::ReadNorFlash;
    use static_cell::StaticCell;

    bind_interrupts!(struct FlashIrqs {
        FLASH => flash::InterruptHandler;
    });

    #[test]
    fn it_works() {
        assert!(true);
    }

    #[test]
    fn storage() {
        // Initialize MCU peripherals
        let p = init(Default::default());

        // Create flash driver (blocking mode)
        let mut flash = Flash::new(p.FLASH, FlashIrqs);

        let result = block_on(Storage::init(flash));
        let Ok(mut _storage) = result else {
            panic!("failed to initialize");
        };
    }

    /*
    // Adjust depending on your chip flash layout
    const TEST_ADDR: u32 = 0x0800_0000; // start of flash
    const TEST_OFFSET: u32 = 0;
    const LEN: usize = 16;

    #[test]
    fn read_flash() {
        // Initialize MCU peripherals
        let p = init(Default::default());

        // Create flash driver (blocking mode)
        let mut flash = Flash::new_blocking(p.FLASH);

        let mut buf = [0u8; LEN];

        // Read flash
        flash.read(TEST_OFFSET, &mut buf).unwrap();

        info!("Flash data: {:x}", buf);

        // Basic sanity check: not all 0xFF (erased) or all 0x00
        let all_ff = buf.iter().all(|&b| b == 0xFF);
        let all_zero = buf.iter().all(|&b| b == 0x00);

        assert!(!(all_ff && all_zero));
    }
    */

    /*
    #[test]
    fn test_async() {
        let p = init(Default::default());

        static EXECUTOR: StaticCell<Executor> = StaticCell::new();
        let executor = EXECUTOR.init(Executor::new());

        let flash = Flash::new_blocking(p.FLASH);

        executor.run(|spawner| {
            spawner.spawn(async_task(flash).unwrap());
        });
    }
    */

    /*
    #[test]
    fn test_async_return() {
        let result = block_on(crate::read_something());
        assert_eq!(result, 42);
    }
    */
}
