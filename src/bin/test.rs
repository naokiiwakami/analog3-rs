#![no_std]
#![no_main]

use analog3::{
    definitions::Value,
    storage::{LAST_SEQ_NUMBER, METADATA_OFFSET, PAGE_0, PAGE_1, PAGE_SIZE, Storage},
};
use defmt::{assert_eq, info};
use defmt_rtt as _;
use defmt_test as _;
use embassy_futures::block_on;
use embassy_stm32 as _;
use embassy_stm32::flash::Flash;
use heapless::String;
use panic_probe as _;

/// Clear all storage areas in the flash memory
fn factory_reset<'a>(flash: &'a mut Flash<'static>) {
    block_on(flash.erase(PAGE_0, PAGE_0 + PAGE_SIZE as u32)).unwrap();
    block_on(flash.erase(PAGE_1, PAGE_1 + PAGE_SIZE as u32)).unwrap();
}

/// Factory reset flash and create storage
fn init<'a>(flash: &'a mut Flash<'static>) -> Storage<'a> {
    factory_reset(flash);

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

/// Test case: Read and write U8 data
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
        u8_verify_final(&mut storage, address0);
    }

    // rebuild storage and verify again
    {
        let mut rebuilt = build_storage(flash);
        u8_verify_final(&mut rebuilt, address0);
    }
    info!("PASS: u8_read_write");
}

/// Verify all the written data in the U8 test
fn u8_verify_final<'a>(storage: &'a mut Storage, address0: u16) {
    let value0 = storage.load_u8(address0).unwrap();
    assert_eq!(value0, 0x1f);
    let value1 = storage.load_u8(address0 + 1).unwrap();
    assert_eq!(value1, 0xca);
    for i in 0..16 {
        let address = address0 + i + 2;
        let value = storage.load_u8(address).unwrap();
        assert_eq!(value, (i * 10) as u8);
    }
}

/// Test case: Read and write U16 data
fn u16_read_write<'a>(flash: &'a mut Flash<'static>) {
    let address0 = 0x10;
    let address1 = 0x35;
    {
        let mut storage = init(flash);

        // The value is unset after the factory reset
        let Ok(value) = storage.load_u16(address0) else {
            panic!("failed to load");
        };
        assert_eq!(value, u16::MAX);

        // Save the initial data
        block_on(storage.save(address0, Value::U16(0xcafe))).unwrap();

        let value = storage.load_u16(address0).unwrap();
        assert_eq!(value, 0xcafe);

        // Overwrite
        block_on(storage.save(address0, Value::U16(0xbeef))).unwrap();
        let value = storage.load_u16(address0).unwrap();
        assert_eq!(value, 0xbeef);

        // Save another u8 to the next.
        block_on(storage.save(address0 + 2, Value::U16(0xface))).unwrap();
        let value0 = storage.load_u16(address0).unwrap();
        assert_eq!(value0, 0xbeef);
        let value1 = storage.load_u16(address0 + 2).unwrap();
        assert_eq!(value1, 0xface);

        // saving a multi byte value across boundary is not recommended,
        // but it should be done properly if requested
        block_on(storage.save(address1, Value::U16(0xba5e))).unwrap();
        let value = storage.load_u16(address1).unwrap();
        assert_eq!(value, 0xba5e);
        block_on(storage.save(address1 + 2, Value::U16(0xba11))).unwrap();
        let value = storage.load_u16(address1 + 2).unwrap();
        assert_eq!(value, 0xba11);
        block_on(storage.save(address1 + 4, Value::U16(0xba7))).unwrap();
        let value = storage.load_u16(address1 + 4).unwrap();
        assert_eq!(value, 0xba7);

        // TODO: try the opposite sequence

        // Write and read 16 U8 items next to it
        for i in 0..0xf {
            let address = address0 + 4 + i * 2;
            let mut value = 0u16;
            for _ in 0..4 {
                value <<= 4;
                value += i as u16;
            }
            block_on(storage.save(address, Value::U16(value))).unwrap();
        }
        // verify written data
        u16_verify_final(&mut storage, address0, address1);

        info!("PASS: u16_read_write");
    }

    // rebuild storage and verify again
    {
        let mut rebuilt = build_storage(flash);
        u16_verify_final(&mut rebuilt, address0, address1);
    }
}

/// Verify all the written data in the U16 test
fn u16_verify_final<'a>(storage: &'a mut Storage, address0: u16, address1: u16) {
    let value = storage.load_u16(address0).unwrap();
    assert_eq!(value, 0xbeef);
    let value = storage.load_u16(address0 + 2).unwrap();
    assert_eq!(value, 0xface);

    let value = storage.load_u16(address1).unwrap();
    assert_eq!(value, 0xba5e);
    let value = storage.load_u16(address1 + 2).unwrap();
    assert_eq!(value, 0xba11);
    let value = storage.load_u16(address1 + 4).unwrap();
    assert_eq!(value, 0xba7);

    for i in 0..0xf {
        let address = address0 + 4 + i * 2;
        let retrieved = storage.load_u16(address).unwrap();
        let mut expected = 0u16;
        for _ in 0..4 {
            expected <<= 4;
            expected += i as u16;
        }
        assert_eq!(retrieved, expected);
    }
}

/// Test case: Read and write TEXT data
fn text_read_write<'a>(flash: &'a mut Flash<'static>) {
    let address0 = 0x50;
    let mut storage = init(flash);

    // string data should be empty initially
    match storage.load_text(address0) {
        Ok(text) => {
            assert_eq!(text.len(), 0);
        }
        Err(e) => {
            panic!("error: {:?}", e);
        }
    }

    // write a string
    block_on(storage.save(
        address0,
        Value::Text(String::try_from("Hello World").unwrap()),
    ))
    .unwrap();
    match storage.load_text(address0) {
        Ok(text) => {
            assert_eq!(text.as_str(), "Hello World");
        }
        Err(e) => {
            panic!("error: {:?}", e);
        }
    };

    // overwrite it
    block_on(storage.save(
        address0,
        Value::Text(String::try_from("Hi there!").unwrap()),
    ))
    .unwrap();
    let text = storage.load_text(address0).unwrap();
    assert_eq!(text.as_str(), "Hi there!");

    // try breaking str
    block_on(storage.save(address0, Value::U8(156))).unwrap();
    let text = storage.load_text(address0).unwrap();
    assert_eq!(text.len(), 0);

    // restore
    block_on(storage.save(address0, Value::Text(String::try_from("restored").unwrap()))).unwrap();
    let text = storage.load_text(address0).unwrap();
    assert_eq!(text.as_str(), "restored");

    // try zero length
    block_on(storage.save(address0, Value::Text(String::try_from("").unwrap()))).unwrap();
    let text = storage.load_text(address0).unwrap();
    assert_eq!(text.len(), 0);

    info!("PASS: text_read_write");
}

fn wrap_sequence_number<'a>(flash: &'a mut Flash<'static>) {
    let address0 = 0x60;
    // proceed sequence number forcefully
    {
        factory_reset(flash);
        let mut row = [u8::MAX; 8];
        row[..2].copy_from_slice(&(LAST_SEQ_NUMBER - 2).to_le_bytes());
        block_on(flash.write(PAGE_1 + METADATA_OFFSET, &row)).unwrap();
        row[..2].copy_from_slice(&(LAST_SEQ_NUMBER - 1).to_le_bytes());
        block_on(flash.write(PAGE_0 + METADATA_OFFSET, &row)).unwrap();
    }

    // build storage and write several times
    for i in 0..5 {
        {
            let mut storage = build_storage(flash);
            block_on(storage.save(address0 + i * 2, Value::U16(i))).unwrap();
            assert_eq!(storage.load_u16(address0 + i * 2).unwrap(), i);
        }
    }

    // final verification
    let mut storage = build_storage(flash);
    for i in 0..5 {
        assert_eq!(storage.load_u16(address0 + i * 2).unwrap(), i);
    }

    info!("PASS: wrap_sequence_number");
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
        crate::u16_read_write(&mut flash);
        crate::text_read_write(&mut flash);
        crate::wrap_sequence_number(&mut flash);
    }
}
