#[derive(Copy)]
pub enum Parity {
    EVEN,
    ODD,
    FORCE0,
    FORCE1,
    NONE,
    MULTIDROP
}

#[derive(Copy)]
pub struct UARTParams {
    // Parity and stop bits should both be enums.
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity
}

pub trait UART {
    fn init(&mut self, params: UARTParams);
    fn send_byte(&mut self, byte: u8);
    fn toggle_rx(&mut self, enable: bool);
    fn toggle_tx(&mut self, enable: bool);
}
