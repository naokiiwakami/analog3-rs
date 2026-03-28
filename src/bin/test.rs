#![no_std]
#![no_main]

use analog3::{
    definitions::Value,
    storage::{PAGE_0, PAGE_1, PAGE_SIZE, Storage},
};
use defmt::assert_eq;
use defmt_rtt as _;
use defmt_test as _;
use embassy_futures::block_on;
use embassy_stm32 as _;
use embassy_stm32::flash::Flash;
use panic_probe as _;

/// Factory reset flash and create storage
fn init<'a>(flash: &'a mut Flash<'static>) -> Storage<'a> {
    // info!(
    //     "erasing page 0 ({:#x} - {:#x})",
    //     PAGE_0,
    //     PAGE_0 + PAGE_SIZE as u32
    // );
    block_on(flash.erase(PAGE_0, PAGE_0 + PAGE_SIZE as u32)).unwrap();
    // info!(
    //     "erasing page 1 ({:#x} - {:#x})",
    //     PAGE_1,
    //     PAGE_1 + PAGE_SIZE as u32
    // );
    block_on(flash.erase(PAGE_1, PAGE_1 + PAGE_SIZE as u32)).unwrap();

    let result = block_on(Storage::init(flash));
    let Ok(storage) = result else {
        panic!("failed to initialize");
    };
    storage
}

/// Build storage without factory reset the flash.
fn build_storage<'a>(flash: &'a mut Flash<'static>) -> Storage<'a> {
    let result = block_on(Storage::init(flash));
    let Ok(storage) = result else {
        panic!("failed to initialize");
    };
    storage
}

fn u8_read_write<'a>(flash: &'a mut Flash<'static>) {
    let address0 = 0x30;

    {
        let mut storage = init(flash);

        // The value is unset after the factory reset
        let Ok(value) = storage.load_u8(address0) else {
            panic!("failed to load");
        };
        assert_eq!(value, u8::MAX);

        // Save the initial data
        block_on(storage.save(address0, Value::U8(0xf1))).unwrap();

        let value = storage.load_u8(address0).unwrap();
        assert_eq!(value, 0xf1);

        // Overwrite
        block_on(storage.save(address0, Value::U8(0x1f))).unwrap();
        let value = storage.load_u8(address0).unwrap();
        assert_eq!(value, 0x1f);

        // Save another u8 to the next.
        block_on(storage.save(address0 + 1, Value::U8(0xca))).unwrap();
        let value0 = storage.load_u8(address0).unwrap();
        assert_eq!(value0, 0x1f);
        let value1 = storage.load_u8(address0 + 1).unwrap();
        assert_eq!(value1, 0xca);

        // Write and read 16 U8 items next to it
        for i in 0..16 {
            let address = address0 + 2 + i;
            let value = (i * 10) as u8;
            block_on(storage.save(address, Value::U8(value))).unwrap();
        }
        // verify written data
        // u8_verify_final(&mut storage, address0);
        let value0 = storage.load_u8(address0).unwrap();
        assert_eq!(value0, 0x1f);
        let value1 = storage.load_u8(address0 + 1).unwrap();
        assert_eq!(value1, 0xca);
        for i in 0..16 {
            let address = address0 + 2 + i;
            let value = storage.load_u8(address).unwrap();
            assert_eq!(value, (i * 10) as u8);
        }
    }

    // rebuild storage and verify again
    {
        let mut storage = build_storage(flash);
        // u8_verify_final(&mut storage, address0);
        let value0 = storage.load_u8(address0).unwrap();
        assert_eq!(value0, 0x1f);
        let value1 = storage.load_u8(address0 + 1).unwrap();
        assert_eq!(value1, 0xca);
        for i in 0..16 {
            let address = address0 + 2 + i;
            let value = storage.load_u8(address).unwrap();
            assert_eq!(value, (i * 10) as u8);
        }
    }
}

fn u8_verify_final<'a>(storage: &'a mut Storage, address0: u16) {
    let value0 = storage.load_u8(address0).unwrap();
    assert_eq!(value0, 0x1f);
    let value1 = storage.load_u8(address0 + 1).unwrap();
    assert_eq!(value1, 0xca);
    for i in 0..16 {
        let address = address0 + i;
        let value = storage.load_u8(address).unwrap();
        assert_eq!(value, (i * 10) as u8);
    }
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    // use defmt::*;
    use embassy_stm32::bind_interrupts;
    use embassy_stm32::{
        flash::{self, Flash},
        init,
    };

    bind_interrupts!(struct FlashIrqs {
        FLASH => flash::InterruptHandler;
    });

    #[test]
    fn storage() {
        // Initialize MCU peripherals
        let p = init(Default::default());

        // Create flash driver (blocking mode)
        let mut flash = Flash::new(p.FLASH, FlashIrqs);

        crate::u8_read_write(&mut flash);

        /*
        match storage.load_text(0x10) {
            Ok(text) => {
                info!("{}", text.as_str());
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }*/
    }
}
