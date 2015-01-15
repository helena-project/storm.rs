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
fn init_console() {
    use platform::sam4l::gpio;
    use platform::sam4l::usart;
    use hil::gpio::*;
    use hil::uart;
    use platform::sam4l::pm;
    let uart_3 = usart::USART::new(usart::BaseAddr::USART3);

    let p1 = gpio::Pin {bus : gpio::Port::PORT1, pin : 9};
    p1.set_peripheral_function(PeripheralFunction::A);
    let p2 = gpio::Pin {bus : gpio::Port::PORT1, pin : 10};
    p2.set_peripheral_function(PeripheralFunction::A);

    pm::enable_pba_clock(11); // USART3 clock

    let mut console = drivers::uart::console::init(uart_3,
        drivers::uart::console::InitParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: uart::Parity::NONE
        }
    );

    console.writeln("Hi there.");
    console.write("Hello thar!");
    console.write(" I'm the captain!");
}
// End of mock UART usage

#[no_mangle]
pub extern fn main() {
    // use platform::sam4l::gpio::*;
    // use platform::sam4l::usart::kstdio::*;
    // use platform::sam4l::{spi, pm};
    // use platform::sam4l::pm::*;
    use task;
    use task::Task::*;

    // use drivers::flash_attr::FlashAttr;

    // kstdio_init();

    init_console();

    /*
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
    } */

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

