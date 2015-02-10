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
    // Here's an ideal platform + device tree. It's a bit complicated, mostly
    // owing to the locations + sublocations.
    // platform_tree!(sam4l,
    //     gpiopin@[1..3][0..32]: gpio::GPIOPin {
    //         function: None
    //     }

    //     // Overriding function on pins 9 and 10 on port 1
    //     gpiopin@1.[9..11]: gpio::GPIOPin {
    //         function: Some(gpio::PeripheralFunction::A)
    //     }

    //     usart@[0..4]: usart::USART;
    // );

    // device_tree!(
    //     console: uart::Console(USART@3) {
    //         baud_rate: 115200,
    //         data_bits: 8,
    //         parity: Parity::None
    //     }

    //     led: gpio::LED(GPIOPin@2.10) {
    //         start_status: LEDStatus::On
    //     }
    // );

    device_tree!(
        first_led: gpio::LED(GPIOPin@10) {
            start_status: LEDStatus::On
        }

        console: uart::Console(UART@3) {
            baud_rate: 115200,
            data_bits: 8,
            parity: Parity::None
        }
    );

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

