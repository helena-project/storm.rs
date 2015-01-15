#![no_main]
#![no_std]
#![allow(dead_code, unstable)]
#![feature(asm)]

extern crate core;
extern crate drivers;
extern crate platform;
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

static mut PROCESS_STACK : [usize; 4096] = [0; 4096];

// Mock UART Usage

struct MyUart;
impl hil::UART for MyUart {
    fn send(&self, thing: &str) {
        use platform::sam4l::usart::kstdio::*;
        kprint(thing);
    }
}

fn init_console() {
    let uart_1 = MyUart;
    let console = drivers::uart::console::init(uart_1,
        drivers::uart::console::InitParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: false,
            stop_bits: 1
        }
    );

    console.speak("Hi there.\n");
    console.speak("Hello thar!\n");
}

// End of mock UART usage

#[no_mangle]
pub extern fn main() {
    use platform::sam4l::gpio::*;
    use platform::sam4l::usart::kstdio::*;
    use platform::sam4l::{spi, pm};
    use platform::sam4l::pm::*;
    use task;
    use task::Task::*;

    use drivers::flash_attr::FlashAttr;

    kstdio_init();

    init_console();

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

    task::Task::UserTask(apps::blinkapp::initialize as usize).post();

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

