#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs, asm, slicing_syntax)]

extern crate core;
extern crate drivers;
extern crate hal;
extern crate support;

extern crate apps;

mod std {
    pub use core::*;
}

mod task;
mod timer;
mod ringbuf;
pub mod syscall;

static mut PROCESS_STACK : [uint,..4096] = [0,..4096];

#[no_mangle]
pub extern fn main() -> int {
    use core::option::Option::*;
    use hal::gpio::*;
    use hal::usart::kstdio::*;
    use hal::pm;
    use hal::pm::*;
    use hal::spi;
    use task;
    use task::Task::*;
    use timer;

    use drivers::flash_attr::FlashAttr;

    kstdio_init();

    {
        pm::enable_pba_clock(1); // SPI clock
        spi::set_mode(spi::MSTR::Master, spi::PS::Variable,
                      spi::RXFIFO::Disable, spi::MODFAULT::Disable);
        spi::enable();
        let mut flash_spi = spi::SPI {cs: 0};
        let mut flash_cs = Pin {bus: Port::PORT2, pin: 3};
        let mut miso = Pin {bus: Port::PORT2, pin: 4};
        let mut mosi = Pin {bus: Port::PORT2, pin: 5};
        let mut sclk = Pin {bus: Port::PORT2, pin: 6};
        let flash_attr = FlashAttr::initialize(&mut flash_spi, &mut flash_cs,
                                               &mut miso, &mut mosi, &mut sclk);

        if flash_attr.do_attr("welcome", |c| { kputc(c as char)}) {
            kputc('\n');
        } else {
            kprint("Welcome to the Tock OS!\n");
        }
    }

    unsafe {
        syscall::DRIVERS[0] = timer::set_user_alarm;
        syscall::NUM_DRIVERS = 1;
    }

    timer::setup();

    unsafe {
        task::setup();
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

