#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(asm,core,core,plugin)]

#[plugin] #[no_link]
extern crate plugins;
extern crate core;
extern crate drivers;
extern crate platform;
extern crate hil;
extern crate support;

use core::prelude::*;

mod std {
    pub use core::*;
}

pub mod config;
mod task;
mod ringbuf;
pub mod syscall;

static mut PSTACKS: [[usize; 256]; 16] = [[0; 256]; 16];

extern {
    static _sapps: u32;
    static _eapps: u32;
}

unsafe fn schedule_external_apps() {
    let (start_ptr, end_ptr) = (&_sapps as *const u32, &_eapps as *const u32);

    let mut ptr = start_ptr;
    while ptr < end_ptr {
        task::Task::UserTask(*ptr as usize).post();
        ptr = ptr.offset(1);
    }
}

fn launch_task(task: task::Task) {
    match task {
        task::Task::UserTask(task_addr) => unsafe {
            syscall::switch_to_user(task_addr, &mut PSTACKS[0][255]);
        },
        task::Task::KernelTask(task) => {
            task();
        }
    }
}

#[no_mangle]
pub extern fn main() {
    config_tree!(
        platform {sam4l,
            gpiopin@[41..43]: gpio::GPIOPin {
                port: GPIOPort::GPIO1,
                function: ::Some(PeripheralFunction::A)
            }

            gpiopin@[64..96]: gpio::GPIOPin {
                port: GPIOPort::GPIO2,
                function: ::None
            }

            uart@[0..4]: usart::USART;
        }

        devices {
            first_led: gpio::LED(GPIOPin@74) {
                start_status: LEDStatus::On
            }

            console: uart::Console(UART@3) {
                baud_rate: 115200,
                data_bits: 8,
                parity: Parity::None
            }
        }
    );

    // TODO: Sublocations? IE: gpiopin@1.[32..64], or gpiopin@[1..3][0..32];
    // platform_tree!(sam4l,
    //     gpiopin@[41..43]: gpio::GPIOPin {
    //         port: GPIOPort::GPIO1,
    //         function: ::Some(PeripheralFunction::A)
    //     }

    //     gpiopin@[64..96]: gpio::GPIOPin {
    //         port: GPIOPort::GPIO2,
    //         function: ::None
    //     }

    //     usart@0: usart::USART;
    //     usart@[1..4]: usart::USART;
    // );

    // device_tree! {
    //     first_led: gpio::LED(GPIOPin@10) {
    //         start_status: LEDStatus::On
    //     }

    //     console: uart::Console(UART@3) {
    //         baud_rate: 115200,
    //         data_bits: 8,
    //         parity: Parity::None
    //     }
    // };

    unsafe {
        task::setup();
        config::config();
        schedule_external_apps();
    }

    loop {
        match unsafe { task::dequeue() } {
            None => {
                support::wfi(); // Sleep!
            },
            Some(task) => {
                launch_task(task);
            }
        }
    }
}

