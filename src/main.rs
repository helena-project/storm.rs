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
mod resource;

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

//static mut INTS : [bool; 100] = [false; 100];
static mut USART3_INT : bool = false;
static mut AST_ALARM_INT : bool = false;

#[no_mangle]
pub extern fn main() {
    use hil::uart::UART;
    use hil::timer::Timer;

    let usart3 = resource::Resource::new(conf::init_console());
    usart3.with(|u| {
        u.init(hil::uart::UARTParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: hil::uart::Parity::None
        });

        u.toggle_rx(true);
        u.toggle_tx(true);

        u.enable_rx_interrupts();
    });


    let ast = resource::Resource::new(conf::init_ast());
    ast.with(|ast| {
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
    });


    unsafe {
        main_loop(usart3.borrow_mut(), ast.borrow_mut());
    }

}

#[inline(never)]
fn main_loop<F: Fn()>(usart3: &mut platform::sam4l::usart::USART,
             ast: &mut platform::sam4l::ast::Ast<F>) {

    loop {
        use platform::sam4l::nvic;
        use hil::uart::UART;

        // Service interrupts
        if unsafe { intrinsics::volatile_load(&USART3_INT) } {
            usart3.interrupt_fired();
            nvic::enable(nvic::NvicIdx::USART3);
        }

        if unsafe { intrinsics::volatile_load(&AST_ALARM_INT) } {
            ast.interrupt_fired();
            usart3.send_byte('h' as u8);
            nvic::enable(nvic::NvicIdx::ASTALARM);
        }

        // Run tasks in task queue

        // Sleep if nothing left to do
        unsafe {
            use core::intrinsics::volatile_load;
            support::atomic(|| {
                if volatile_load(&USART3_INT) || volatile_load(&AST_ALARM_INT) {
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


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn AST_ALARM_Handler() {
    use platform::sam4l::*;
    AST_ALARM_INT = true;
    nvic::disable(nvic::NvicIdx::ASTALARM);
}

