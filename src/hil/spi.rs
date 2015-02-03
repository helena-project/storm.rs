#[derive(Copy)]
pub enum Mode { // Mode is encoded as CPOL in bit 0 and NCPHA in bit 1
    Mode0 = 2,  // CPOL == 0, NCPHA = 1
    Mode1 = 0,  // CPOL == 0, NCPHA = 0
    Mode2 = 3,  // CPOL == 1, NCPHA = 1
    Mode3 = 1   // CPOL == 1, NCPHA = 0
}

pub trait SPIMaster {
    fn enable(&mut self);
    fn disable(&mut self);
    fn set_baud_rate(&mut self, divisor: u8);
    fn set_mode(&mut self, Mode);
    fn write_read(&mut self, u16, bool) -> u16;
}

