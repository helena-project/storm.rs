#![crate_name = "apps"]
#![crate_type = "rlib"]
#![allow(unstable)]
#![no_std]

extern crate core;
extern crate platform;
extern crate hil;

use core::prelude::*;

pub mod blink;

#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num: usize, arg1: usize, arg2: usize) -> isize;
    fn __command(driver_num: usize, arg1: usize, arg2: usize) -> isize;
    fn __wait() -> isize;
}

fn writeln(line: &str) {
    unsafe {
        for byte in line.bytes() {
            __command(0, byte as usize, 0);
        }
        __command(0, '\n' as usize, 0);
    }
}

fn toggle_led() {
    unsafe {
        __command(1, 0, 0);
    }
}
