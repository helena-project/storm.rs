#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs)]

extern crate core;
extern crate hal;
extern crate support;

use core::option::*;

mod task;
mod timer;
mod init;
mod ringbuf;

#[no_mangle]
pub extern fn main() -> int {
    unsafe {
        task::setup();
        task::post(init::init);
        loop {
          match task::dequeue() {
            None => {
                support::wfi() // Sleep!
            },
            Some(task::Task(task)) => { task() }
          }
        }
    }
}

