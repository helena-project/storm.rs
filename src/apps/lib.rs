#![crate_name = "apps"]
#![crate_type = "rlib"]
#![allow(unstable)]
#![no_std]

extern crate core;
extern crate platform;
extern crate hil;

use core::prelude::*;

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

pub mod blinkapp {
    static mut count: usize = 0;

    #[inline(never)]
    pub fn initialize() {
        super::toggle_led();
        super::writeln("I'm in the app!");

        unsafe {
            super::__subscribe(0, 1 << 15, timer_fired as usize);
            super::__wait();
        }
    }

    #[inline(never)]
    pub fn timer_fired() {
        super::toggle_led();

        unsafe {
            count += 1;
            if count % 10 == 0 {
                super::writeln("Timer fired 10 times");
            }
        }

        unsafe {
            super::__subscribe(0, 1 << 15, timer_fired as usize);
            super::__wait();
        }
    }
}

