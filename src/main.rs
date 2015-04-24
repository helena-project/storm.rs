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

static mut INTERRUPTS : [bool; 64] = [false; 64];

unsafe fn interrupts_empty() -> bool {
    for interrupt_num in INTERRUPTS.iter() {
        if *interrupt_num {
            return false;
        }
    }
    return true;
}

// TODO(alevy): switch to using intrinsics::atomic_xchg when compiler error
// resolved
unsafe fn atomic_xchg<T>(dst: *mut T, src: T) -> T {
    let res = intrinsics::volatile_load(dst);
    *dst = src;
    return res;
}

#[no_mangle]
pub extern fn main() {
    use hil::uart::UART;
    use platform::sam4l::usart;

    let mut usart3 = conf::init_console();
    usart3.init(hil::uart::UARTParams {
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    usart3.toggle_rx(true);
    usart3.toggle_tx(true);

    let c = usart3.read_byte();
    usart3.send_byte(c as u8);
    usart3.send_byte('g' as u8);
    let c = usart3.read_byte();
    usart3.send_byte(c as u8);
    usart3.send_byte('g' as u8);

    usart3.send_byte('h' as u8);


/*    {
        usart3.init(hil::uart::UARTParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: hil::uart::Parity::None
        });


        usart3.toggle_tx(true);
        usart3.toggle_rx(true);

        usart3.reset_rx();
    }*/

    //let c = usart3.read_byte();
    //let c = usart3.read_byte();
    //usart3.send_byte((c % 10 + 48) as u8);
    //usart3.send_byte((c / 10 % 10 + 48) as u8);

    loop {

        unsafe {
            for i in range(0, INTERRUPTS.len()) {
                let interrupt_num = &mut INTERRUPTS[i];
                let fired = atomic_xchg(interrupt_num, false);
                if fired {
                    match i {
                        1 => {
                           usart3.interrupt_fired(); 
                        },
                        _ => {}
                    }
                }
            }

            /*support::atomic(|| {
                if interrupts_empty() {
                    support::wfi()
                }
            });*/
        }
    }
}

/*#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn USART3_Handler() {
    use platform::sam4l::*;
    use hil::uart::UART;
    INTERRUPTS[1] = true;
    let mut usart_3 = usart::USART::new(usart::USARTParams {
        location: usart::Location::USART3
    });
    usart_3.send_byte('h' as u8);
}*/

