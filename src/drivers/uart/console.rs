use hil::UART;

pub struct InitParams {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: bool,
    pub stop_bits: u8
}

pub struct Console<T: UART> {
    uart: T,
    params: InitParams
}

impl<T: UART> Console<T> {
    pub fn speak(&self, thing: &str) {
        self.uart.send(thing);
    }
}

pub fn init<T: UART>(uart: T, params: InitParams) -> Console<T> {
    Console {
        uart: uart,
        params: params
    }
}
