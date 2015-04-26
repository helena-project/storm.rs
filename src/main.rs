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

static mut ASTALARM_INT : bool = false;
static mut USART_INT : bool = false;

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

#[inline(never)]
fn main_loop<F: Fn()>(mut usart3: platform::sam4l::usart::USART,
             mut ast: platform::sam4l::ast::Ast<F>) {
    use platform::sam4l::*;
    use hil::timer::Timer;
    use hil::gpio::*;

    let systick_csr : &mut u32 = unsafe { intrinsics::transmute(0xE000E010) };
    let systick_rvr : &mut u32 = unsafe { intrinsics::transmute(0xE000E014) };
    let systick_val : &mut u32 = unsafe { intrinsics::transmute(0xE000E018) };

    unsafe {
        intrinsics::volatile_store(systick_rvr, 0xffffff);
        intrinsics::volatile_store(systick_csr, (1 << 2 | 1));
    }

    let mut start = 0;

    loop {

        unsafe {
            if intrinsics::volatile_load(&USART_INT) {
                usart3.interrupt_fired();
                USART_INT = false;
                nvic::enable(nvic::NvicIdx::USART3);
            }

            if intrinsics::volatile_load(&ASTALARM_INT) {
                let end = intrinsics::volatile_load(systick_val);
                let fin = start - end;
                print_num(&mut usart3, fin);
                start = intrinsics::volatile_load(systick_val);
                ast.interrupt_fired();
                ASTALARM_INT = false;
                nvic::enable(nvic::NvicIdx::ASTALARM);
            }
        }
        unsafe {
            support::atomic(|| {
                if !intrinsics::volatile_load(&USART_INT) {
                    support::wfi();
                    start = intrinsics::volatile_load(systick_val);
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
    use platform::sam4l::*;
    ASTALARM_INT = true;
    nvic::disable(nvic::NvicIdx::ASTALARM);
}

pub fn print_num(console: &mut platform::sam4l::usart::USART, val: u32) {
    use hil::uart::*;
    let mut num = val;
    let mut first = true;
    let mut d = 1;
    let base = 10;

    while num/d >= base {
        d *= base;
    }

    while d != 0 {
        let digit = num / d;
        num %= d;
        d /= base;

        if !first || digit > 0 || d==0 {
            console.send_byte((digit + 0x30) as u8);
            first = false;
        }
    }

    console.send_byte('\n' as u8);
}
