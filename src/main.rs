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
mod init;
mod ringbuf;
mod usart;
mod pm;
pub mod support;

#[no_mangle]
pub extern fn main() -> int {
    task::post(init::init);
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
