use embassy_stm32::i2c::{Error, I2c, Master};
use embassy_stm32::mode::Async;

/// Device address
#[repr(u8)]
pub enum Address {
    A0 = 0b1100000,
    A1 = 0b1100001,
    A2 = 0b1100010,
    A3 = 0b1100011,
    A4 = 0b1100100,
    A5 = 0b1100101,
    A6 = 0b1100110,
    A7 = 0b1100111,
}

/// Resistor Ladder Voltage Reference selection.
#[repr(u8)]
pub enum Vrl {
    /// Vdd (Unbuffered).
    Vdd = 0b00,
    /// Vref pin (Unbuffered).
    VrefUnbuffered = 0b10,
    /// Vref pin (Buffered).
    VrefBuffered = 0b11,
}

/// Power-Down selection.
#[repr(u8)]
pub enum PowerDown {
    /// Not Powered Down (Normal operation).
    NormalOperation = 0b00,
    /// Powered Down - Vout is loaded with 1 kOhm resistor to ground.
    PowerDown1KOhm = 0b01,
    /// Powered Down - Vout is loaded with 100 kOhm resistor to ground.
    PowerDown100KOhm = 0b10,
    /// Powered Down - Vout is loaded with 500 kOhm resistor to ground.
    PowerDown500KOhm = 0b11,
}

/// Gain selection.
#[repr(u8)]
pub enum Gain {
    /// Gain of 1.
    One = 0,
    /// Gain of 2. Not applicable when Vdd is used as Vrl
    Two = 1,
}

#[non_exhaustive]
pub struct Config {
    pub vrl: Vrl,
    pub pd: PowerDown,
    pub g: Gain,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vrl: Vrl::Vdd,
            pd: PowerDown::NormalOperation,
            g: Gain::One,
        }
    }
}

/// Driver for a single MCP47x6 DAC device on a shared I2C bus.
pub struct Mcp47x6 {
    address: u8,
}

impl Mcp47x6 {
    pub const fn new(address: Address) -> Self {
        Self {
            address: address as u8,
        }
    }

    /// Initialize the MCP47x6 device by writing the volatile configuration registers.
    pub async fn initialize(
        &self,
        i2c: &mut I2c<'static, Async, Master>,
        config: Config,
    ) -> Result<(), Error> {
        let command = 0b010; // write volatile memory
        let vref = config.vrl as u8;
        let pd = config.pd as u8;
        let g = config.g as u8;

        let data = [(command << 5) | (vref << 3) | (pd << 1) | g, 0, 0];

        i2c.write(self.address, &data).await
    }

    /// Update the DAC output using the fast write protocol.
    pub async fn update(
        &self,
        i2c: &mut I2c<'static, Async, Master>,
        value: u16,
    ) -> Result<(), Error> {
        let data = [((value >> 12) & 0x0f) as u8, ((value >> 4) & 0xff) as u8];
        i2c.write(self.address, &data).await
    }
}
