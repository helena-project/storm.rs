#![crate_name = "apps"]
#![crate_type = "rlib"]
#![allow(unstable, unused)]
#![no_std]

extern crate core;

use core::prelude::*;

pub mod blink;

// List of commands
const CMD_PRINTC: usize = 0;
const CMD_TOGGLE_LED: usize = 1;

// List of subscriptions
const SUB_TIMER: usize = 0;

#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num: usize, arg1: usize, arg2: fn());
    fn __command(driver_num: usize, arg1: usize, arg2: usize);
    fn __wait(a: usize, b: usize, c: usize);
}

fn println(line: &str) {
    unsafe {
        for byte in line.bytes() {
            __command(CMD_PRINTC, byte as usize, 0);
        }

        __command(CMD_PRINTC, '\n' as usize, 0);
    }
}

fn toggle_led() {
    unsafe {
        __command(CMD_TOGGLE_LED, 0, 0);
    }
}

fn timer_subscribe(time: usize, f: fn()) {
    unsafe {
        __subscribe(SUB_TIMER, time, f);
    }
}

fn wait() {
    unsafe {
        __wait(0, 0, 0);
    }
}
