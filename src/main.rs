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
mod util;
mod ring_buffer;
mod process;
mod loader;
mod syscall;


#[no_mangle]
pub extern fn main() {
    let mut proc_list = unsafe {
        config::config();
        util::println("Code Started!");

        let mut buf : [u8; 1024] = [0; 1024];
        let mut list = ArrayList::new(8, intrinsics::transmute(&mut buf));
        loader::load_apps(&mut list);
        util::println("After code");
        list
    };

    let subscribe_drivers = unsafe { &syscall::SUBSCRIBE_DRIVERS };
    let cmd_drivers = unsafe { &syscall::CMD_DRIVERS };

    // Circular iterator is temporary. We actually want a run queue so we can
    // sleep when there is no work to be done.
    let mut iter = proc_list.circular_iterator();
    let mut process = iter.next().unwrap();
    loop {
        match process.state {
            process::State::Running => {
                unsafe { process.switch_to(); }
            },
            process::State::Waiting => {
                unsafe {
                    match process.callbacks.dequeue() {
                        None => {
                            process = iter.next().unwrap();
                            continue;
                        },
                        Some(cb) => {
                            process.state = process::State::Running;
                            process.switch_to_callback(cb);
                        }
                    }
                };
            }
        }
        let process_ptr = process as *mut Process as *mut ();
        match process.svc_number() {
            Some(syscall::WAIT) => {
                process.state = process::State::Waiting;
                process.pop_syscall_stack();
                process = iter.next().unwrap();
            },
            Some(syscall::SUBSCRIBE) => {
                let driver = subscribe_drivers[process.r0()];
                let res = driver(process_ptr, process.r1(),process.r2());
                process.set_r0(res);
            },
            Some(syscall::COMMAND) => {
                let driver = cmd_drivers[process.r0()];
                let res = driver(process_ptr, process.r1(), process.r2());
                process.set_r0(res);
            },
            _ => {}
        }
    }
}

