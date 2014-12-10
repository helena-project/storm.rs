#![no_main]
#![no_std]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(lang_items)]
#![feature(intrinsics)]
#![feature(asm)]

extern crate core;

mod gpio;
mod ast;
mod nvic;
mod task;
mod timer;
mod lang_items;
mod ringbuf;
mod usart;
pub mod support;

static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

fn set_led() {
    LED.set();
    timer::set_alarm(1 << 16, set_led);
}

fn clear_led() {
    LED.clear();
    timer::set_alarm(1 << 16, clear_led);
}

fn app_entry() {
    LED.make_output();
    LED.set();

    let uart = usart::USART::UART3;
    uart.init_uart();
    uart.set_baud_rate(38400);
    uart.enable_tx();

    while !uart.tx_ready() {}
    uart.send_byte('a' as u8);

    while !uart.tx_ready() {}
    uart.send_byte('b' as u8);

    LED.clear();

    timer::setup();

    timer::set_alarm(1 << 15, set_led);
    timer::set_alarm(1 << 16, clear_led);
}

#[no_mangle]
pub extern fn main() -> int {
    task::post(app_entry);
    loop {
      use core::option::{None, Some};
      match unsafe { task::dequeue() } {
        None => {
            support::wfi() // Sleep!
        },
        Some(task::Task(task)) => { task() }
      }
    }
}

#[no_mangle]
pub extern fn AST_ALARM_Handler() {
  timer::ast_alarm_handler();
}
