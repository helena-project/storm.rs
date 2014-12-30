#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs, asm, slicing_syntax)]

extern crate core;
extern crate drivers;
extern crate hal;
extern crate hil;
extern crate support;

extern crate apps;

use core::prelude::*;

mod std {
    pub use core::*;
}

mod config;
mod task;
mod ringbuf;
pub mod syscall;

static mut PROCESS_STACK : [uint,..4096] = [0,..4096];

#[no_mangle]
pub extern fn main() -> int {
    use hal::gpio::*;
    use hal::usart::kstdio::*;
    use hal::pm;
    use hal::pm::*;
    use hal::spi;
    use task;
    use task::Task::*;

    use drivers::flash_attr::FlashAttr;

    kstdio_init();
    spi::set_mode(spi::MSTR::Master, spi::PS::Variable,
                      spi::RXFIFO::Disable, spi::MODFAULT::Disable);
    spi::enable();

    if false {
        pm::enable_pba_clock(1); // SPI clock
        let flash_spi = spi::SPI {cs: 0};
        let flash_cs = Pin {bus: Port::PORT2, pin: 3};
        let miso = Pin {bus: Port::PORT2, pin: 4};
        let mosi = Pin {bus: Port::PORT2, pin: 5};
        let sclk = Pin {bus: Port::PORT2, pin: 6};
        let flash_attr = FlashAttr::initialize(flash_spi, flash_cs,
                                               miso, mosi, sclk);

        if flash_attr.do_attr("welcome", |c| { kputc(c as char)}) {
            kputc('\n');
        } else {
            kprint("Welcome to the Tock OS!\n");
        }
    }

    unsafe {
        task::setup();
        config::config();
    }

    task::Task::UserTask(apps::blinkapp::initialize as uint).post();

    loop {
        match unsafe { task::dequeue() } {
            None => {
                support::wfi(); // Sleep!
            },
            Some(task) => {
                match task {
                    UserTask(task_addr) => unsafe {
                        syscall::switch_to_user(
                            task_addr, &mut PROCESS_STACK[255]);
                    },
                    KernelTask(task) => {
                        task();
                    }
                }
            }
        }
    }
}

