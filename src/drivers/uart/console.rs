use hil::{UART, UARTParams, Parity};
use hil::uart;
use core::prelude::*;

pub struct ConsoleParams {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity
}

pub struct Console<T: UART> {
    uart: T,
    read_callback: Option<fn(u8)>
}

impl<T: UART> Console<T> {
    pub fn new(mut uart: T, params: ConsoleParams) -> Console<T> {
        uart.init(UARTParams {
            baud_rate: params.baud_rate,
            data_bits: params.data_bits,
            parity: params.parity
        });

        uart.enable_tx();
        Console {
            uart: uart,
            read_callback: None
        }
    }

    pub fn putc(&mut self, byte: u8) {
        self.uart.send_byte(byte);
    }

    pub fn read_subscribe(&mut self, callback: fn(u8)) {
        if self.read_callback.is_none() {
            self.uart.enable_tx();
        }

        self.read_callback = Some(callback);
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

impl<T: UART> uart::Reader for Console<T> {

    fn read_done(&mut self, byte: u8) {
        self.putc(byte);
    }
}
