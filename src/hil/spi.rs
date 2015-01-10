use core::prelude::*;

pub enum Mode { // Mode is encoded as CPOL in bit 0 and NCPHA in bit 1
    Mode0 = 2,  // CPOL == 0, NCPHA = 1
    Mode1 = 0,  // CPOL == 0, NCPHA = 0
    Mode2 = 3,  // CPOL == 1, NCPHA = 1
    Mode3 = 1   // CPOL == 1, NCPHA = 0
}

impl Copy for Mode {}

pub trait SPI {
    fn set_baud_rate(&self, divisor : u8);
    fn set_mode(&self, Mode);
    fn write_read(&self, u16, bool) -> u16;
}

