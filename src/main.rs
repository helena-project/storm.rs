#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs, lang_items, asm)]

extern crate core;
extern crate hal;

mod task;
mod timer;
mod lang_items;
mod init;
mod ringbuf;
pub mod support;

#[no_mangle]
pub extern fn main() -> int {
    unsafe {
        task::setup();
        task::post(init::init);
        loop {
          use core::option::{None, Some};
          match task::dequeue() {
            None => {
                support::wfi() // Sleep!
            },
            Some(task::Task(task)) => { task() }
          }
        }
    }
}

