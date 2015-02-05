use hil::{UART, UARTParams, Parity};
use core::prelude::*;

#[derive(Copy)]
pub struct InitParams {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity
}

pub struct Console<T: UART> {
    uart: T,
}

impl<T: UART> Console<T> {
    pub fn putc(&mut self, byte: u8) {
        self.uart.send_byte(byte);
    }

    pub fn write(&mut self, content: &str) {
        for byte in content.bytes() {
            self.putc(byte);
        }
    }

    pub fn writeln(&mut self, content: &str) {
        self.write(content);
        self.putc('\n' as u8);
    }
}

pub fn init<U>(mut uart: U, params: InitParams)
        -> Console<U> where U: UART {
    uart.init(UARTParams {
        baud_rate: params.baud_rate,
        data_bits: params.data_bits,
        parity: params.parity
    });

    uart.toggle_tx(true);
    uart.toggle_rx(false);

    Console {
        uart: uart,
    }
}
