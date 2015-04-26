#![feature(asm,core,core,plugin,no_std)]
#![allow(dead_code)]
#![no_main]
#![no_std]

extern crate core;
extern crate support;
extern crate platform;
extern crate hil;

use core::prelude::*;
use core::intrinsics;

mod std {
    pub use core::*;
}

mod conf {
    use core::prelude::*;
    use platform::sam4l::*;

    pub fn init_console() -> usart::USART {
        let usart_3 = usart::USART::new(usart::USARTParams {
            location: usart::Location::USART3
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

        usart_3
    }

    pub fn init_ast() -> ast::Ast {
        ast::Ast::new()
    }
}

static mut ASTALARM_INT : bool = false;
static mut USART_INT : bool = false;

#[no_mangle]
pub extern fn main() {
    use hil::uart::UART;

    let mut usart3 = conf::init_console();
    usart3.init(hil::uart::UARTParams {
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    usart3.toggle_rx(true);
    usart3.toggle_tx(true);

    usart3.enable_rx_interrupts();

    main_loop(usart3);
}

#[inline(never)]
fn main_loop(mut usart3: platform::sam4l::usart::USART) {
    use platform::sam4l::*;

    loop {

        unsafe {
            if intrinsics::volatile_load(&USART_INT) {
                usart3.interrupt_fired();
                USART_INT = false;
                nvic::enable(nvic::NvicIdx::USART3);
            }

            if intrinsics::volatile_load(&ASTALARM_INT) {
                //usart3.interrupt_fired();
                ASTALARM_INT = false;
                nvic::enable(nvic::NvicIdx::ASTALARM);
            }
        }
        unsafe {
            support::atomic(|| {
                if !intrinsics::volatile_load(&USART_INT) {
                    support::wfi();
                }
            });
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn USART3_Handler() {
    use platform::sam4l::nvic;
    USART_INT = true;
    nvic::disable(nvic::NvicIdx::USART3);
}


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn AST_ALARM_Handler() {
    use platform::sam4l::nvic;
    ASTALARM_INT = true;
    nvic::disable(nvic::NvicIdx::ASTALARM);
}

