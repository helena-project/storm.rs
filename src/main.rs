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
}

static mut USART_INT : bool = false;

/*unsafe fn interrupts_empty() -> bool {
    for interrupt_num in INTERRUPTS.iter() {
        if *interrupt_num {
            return false;
        }
    }
    return true;
}*/

unsafe fn atomic_xchg<T>(dst: *mut T, src: T) -> T {
    let res = intrinsics::volatile_load(dst);
    intrinsics::volatile_store(dst, src);
    return res;
}

#[no_mangle]
pub extern fn main() {
    use platform::sam4l::usart;
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
    use hil::uart::UART;

    loop {

        unsafe {
            if intrinsics::volatile_load(&USART_INT) {
                //let c = usart3.read_byte();
                //usart3.send_byte(c);
                usart3.interrupt_fired(); 
                USART_INT = false;
            }

            /*asm!("cpsid i" :::: "volatile");
            asm!("wfi" :::: "volatile");
            asm!("cpsie i" :::: "volatile");*/
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn USART3_Handler() {
    use platform::sam4l::*;
    use hil::uart::UART;
    let mut usart_3 = usart::USART::new(usart::USARTParams {
        location: usart::Location::USART3
    });
    //let c = usart_3.read_byte();
    //usart_3.send_byte(c);
    if usart_3.rx_ready() {
        USART_INT = true;
        usart_3.disable_rx_interrupts();
    }
}

