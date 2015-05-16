#![feature(asm,core,core,plugin,no_std)]
#![allow(dead_code)]
#![no_main]
#![no_std]

extern crate core;
extern crate drivers;
extern crate support;
extern crate platform;
extern crate hil;

use core::prelude::*;
use core::intrinsics;

pub mod syscall;
mod shared;

mod std {
    pub use core::*;
}


mod conf {
    use core::prelude::*;
    use platform::sam4l::*;
    use shared::Shared;

    pub static mut USART3 : Option<Shared<usart::USART>> = None;
    pub static mut SHELL : Option<Shared<::EchoShell>> = None;

    pub fn init() {
        let usart_3 = usart::USART::new(usart::USARTParams {
            location: usart::Location::USART3,
            client: None
        });

        let mut pin9 = gpio::GPIOPin::new(gpio::GPIOPinParams {
            location: gpio::Location::GPIOPin9,
            port: gpio::GPIOPort::GPIO1,
            function: Some(gpio::PeripheralFunction::A)
        });
        pin9.set_ster();

        let _ = gpio::GPIOPin::new(gpio::GPIOPinParams {
            location: gpio::Location::GPIOPin10,
            port: gpio::GPIOPort::GPIO1,
            function: Some(gpio::PeripheralFunction::A)
        });

        unsafe {
            USART3 = Some(Shared::new(usart_3));
            SHELL = Some(Shared::new(
                    ::EchoShell::new(USART3.as_ref().unwrap().borrow_mut())));
            USART3.as_ref().unwrap().borrow_mut().set_client(SHELL.as_ref().unwrap().borrow_mut());
        }

    }
}

static mut USART3_INT : bool = false;

struct EchoShell {
    uart: &'static mut platform::sam4l::usart::USART,
    buf: [u8; 40],
    cur: usize
}

impl EchoShell {
    pub fn subscribe(&mut self) {
    }
}

impl EchoShell {
    fn new(uart: &'static mut platform::sam4l::usart::USART) -> EchoShell {
        EchoShell {
            uart: uart,
            buf: [0; 40],
            cur: 0
        }
    }
}

impl hil::uart::Reader for EchoShell {
    fn read_done(&mut self, byte : u8) {
        use hil::uart::UART;
        if byte as char == '\n' {
            self.uart.send_byte('\r' as u8);
            self.uart.send_byte('\n' as u8);
            let mut c = (self.cur / 10) % 10 + ('0' as usize);
            self.uart.send_byte(c as u8);
            let mut c = self.cur % 10 + ('0' as usize);
            self.uart.send_byte(c as u8);
            self.uart.send_byte('>' as u8);
            self.cur = 0;
        }
        if (self.cur < self.buf.len()) {
            self.buf[self.cur] = byte;
            self.cur += 1;
        }
    }
}

#[no_mangle]
pub extern fn main() {
    use hil::uart::UART;
    use shared::Shared;

    conf::init();

    let usart3r = unsafe { conf::USART3.as_ref().unwrap() };

    let shell = {
        let usart3 = usart3r.borrow_mut();
        usart3.init(hil::uart::UARTParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: hil::uart::Parity::None
        });

        usart3.enable_rx();
        usart3.enable_tx();

        usart3.enable_rx_interrupts();
        Shared::new(EchoShell::new(usart3))
    };

    loop {
        use platform::sam4l::nvic;
        use hil::uart::UART;

        // Service interrupts
        if unsafe { intrinsics::volatile_load(&USART3_INT) } {
            usart3r.borrow_mut().interrupt_fired();
            nvic::enable(nvic::NvicIdx::USART3);
        }

        // Let's pretend we just got a syscall from an app...
        shell.borrow_mut().subscribe();

        // Sleep if nothing left to do
        unsafe {
            use core::intrinsics::volatile_load;
            support::atomic(|| {
                if volatile_load(&USART3_INT) {
                    return
                }
                support::wfi();
            });
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn USART3_Handler() {
    use platform::sam4l::nvic;
    USART3_INT = true;
    nvic::disable(nvic::NvicIdx::USART3);
}

