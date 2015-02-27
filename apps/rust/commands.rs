use core::prelude::*;

// List of commands
const CMD_PRINTC: usize = 0;
const CMD_TOGGLE_LED: usize = 1;
const CMD_TMP006_READ: usize = 2;

// List of subscriptions
const SUB_TIMER: usize = 0;

#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num: usize, arg1: usize, arg2: fn());
    fn __command(driver_num: usize, arg1: usize, arg2: usize);
    fn __wait(a: usize, b: usize, c: usize);
}

pub fn println(line: &str) {
    unsafe {
        for byte in line.bytes() {
            __command(CMD_PRINTC, byte as usize, 0);
        }

        __command(CMD_PRINTC, '\n' as usize, 0);
    }
}

pub fn toggle_led() {
    unsafe {
        __command(CMD_TOGGLE_LED, 0, 0);
    }
}

pub fn timer_subscribe(time: usize, f: fn()) {
    unsafe {
        __subscribe(SUB_TIMER, time, f);
    }
}

pub fn wait() {
    unsafe {
        __wait(0, 0, 0);
    }
}

pub fn tmp006_read_sync() {
    unsafe {
        __command(CMD_TMP006_READ, 0, 0);
    }
}
