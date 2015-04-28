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

    pub fn init_ast<F: Fn()>() -> ast::Ast<F> {
        let mut ast = ast::Ast::new();
        ast.setup();
        ast
    }
}

static mut INTS : [bool; 100] = [false; 100];

#[no_mangle]
pub extern fn main() {
    use hil::uart::UART;
    use hil::timer::Timer;

    let mut usart3 = conf::init_console();
    usart3.init(hil::uart::UARTParams {
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    usart3.toggle_rx(true);
    usart3.toggle_tx(true);

    usart3.enable_rx_interrupts();

    let mut ast = conf::init_ast();
    ast.set_callback(|| {
        use hil::gpio::GPIOPin;
        use platform::sam4l::*;
        let mut pin_10 = gpio::GPIOPin::new(gpio::GPIOPinParams {
            location: gpio::Location::GPIOPin10,
            port: gpio::GPIOPort::GPIO2,
            function: None
        });

        pin_10.enable_output();
        pin_10.toggle();
    });
    let alm = ast.now() + (1<<15);
    ast.set_alarm(alm);

    main_loop(usart3, ast);
}

fn main_loop<F: Fn()>(mut usart3: platform::sam4l::usart::USART,
             mut ast: platform::sam4l::ast::Ast<F>) {

    let ints_len = unsafe { INTS.len() };
    loop {

        // Service interrupts
        for i in range(0, ints_len) {
            if unsafe { intrinsics::volatile_load(&INTS[i]) } {
                use platform::sam4l::nvic;
                let nvic = match i {
                    0 => {
                        usart3.interrupt_fired();
                        Some(nvic::NvicIdx::USART3)
                    }
                    1 => {
                        ast.interrupt_fired();
                        Some(nvic::NvicIdx::ASTALARM)
                    }
                    _ => None
                };
                unsafe { INTS[i] = false };
                nvic.map(|n| { nvic::enable(n) });
            }
        }

        // Run tasks in task queue

        // Sleep if nothing left to do
        unsafe {
            support::atomic(|| {
                for i in INTS.iter() {
                    if *i {
                        return;
                    }
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
    INTS[0] = true;
    nvic::disable(nvic::NvicIdx::USART3);
}


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn AST_ALARM_Handler() {
    use platform::sam4l::*;
    INTS[1] = true;
    nvic::disable(nvic::NvicIdx::ASTALARM);
}

