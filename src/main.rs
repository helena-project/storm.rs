#![feature(asm,core,core,plugin,no_std)]
#![allow(dead_code)]
#![no_main]
#![no_std]
#![plugin(plugins)]

#[macro_use(panic)]
extern crate core;
extern crate drivers;
extern crate platform;
extern crate hil;
extern crate support;

use core::prelude::*;
use core::intrinsics;

use array_list::ArrayList;
use process::Process;

mod std {
    pub use core::*;
}

mod array_list;
pub mod config;
mod task;
mod ringbuf;
mod process;
mod syscall;

static mut PSTACKS: [[u8; 1024]; 8] = [[0; 1024]; 8];

#[allow(improper_ctypes)]
extern {
    static _sapps: fn();
    static _eapps: fn();
}

unsafe fn schedule_external_apps(proc_arr: &mut ArrayList<Process>) {

    let (start_ptr, end_ptr) = (&_sapps as *const fn(), &_eapps as *const fn());

    let mut ptr = start_ptr;
    while ptr < end_ptr {
        match process::Process::create(*ptr) {
            Err(_) => { break; },
            Ok(process) => {
                if !proc_arr.add(process) {
                    break;
                }
            }
        }
        ptr = ptr.offset(1);
    }
}

#[no_mangle]
pub extern fn main() {
    let mut proc_list = unsafe {
        task::setup();
        config::config();

        let mut buf : [u8; 1024] = [0; 1024];
        let mut list = ArrayList::new(8, intrinsics::transmute(&mut buf));
        schedule_external_apps(&mut list);
        list
    };

    let subscribe_drivers = unsafe { &syscall::SUBSCRIBE_DRIVERS };
    let cmd_drivers = unsafe { &syscall::CMD_DRIVERS };

    let mut iter = proc_list.circular_iterator();
    let mut process = iter.next().unwrap();
    loop {
        unsafe { process.switch_to() };
        match process.svc_number() {
            Some(syscall::WAIT) => {
                process = iter.next().unwrap();
            },
            Some(syscall::SUBSCRIBE) => {
                let res = subscribe_drivers[process.r0()](process.r1(),process.r2());
                process.set_r0(res);
            },
            Some(syscall::COMMAND) => {
                let res = cmd_drivers[process.r0()](process.r1(), process.r2());
                process.set_r0(res);
            },
            _ => {}
        }
    }
}

