#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(asm,core,core)]

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

// fn setup_spi() {
//     use drivers::flash_attr::FlashAttr;
//     spi::set_mode(spi::MSTR::Master, spi::PS::Variable,
//                       spi::RXFIFO::Disable, spi::MODFAULT::Disable);
//     spi::enable();

//     if false {
//         pm::enable_pba_clock(1); // SPI clock
//         let flash_spi = spi::SPI {cs: 0};
//         let flash_cs = Pin {bus: Port::PORT2, pin: 3};
//         let miso = Pin {bus: Port::PORT2, pin: 4};
//         let mosi = Pin {bus: Port::PORT2, pin: 5};
//         let sclk = Pin {bus: Port::PORT2, pin: 6};
//         let flash_attr = FlashAttr::initialize(flash_spi, flash_cs,
//                                                miso, mosi, sclk);

//         if flash_attr.do_attr("welcome", |c| { kputc(c as char)}) {
//             kputc('\n');
//         } else {
//             kprint("Welcome to the Tock OS!\n");
//         }
//     }
// }
