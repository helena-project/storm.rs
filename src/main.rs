#![feature(asm,core,core,plugin,no_std)]
#![allow(dead_code)]
#![no_main]
#![no_std]
#![plugin(plugins)]

extern crate core;
extern crate drivers;
extern crate platform;
extern crate hil;
extern crate support;

use core::prelude::*;

mod std {
    pub use core::*;
}

pub mod config;
mod task;
mod ringbuf;
mod process;
pub mod syscall;

static mut PSTACKS: [[u8; 1024]; 8] = [[0; 1024]; 8];

#[allow(improper_ctypes)]
extern {
    static _sapps: fn();
    static _eapps: fn();
}

unsafe fn schedule_external_apps() {

    let (start_ptr, end_ptr) = (&_sapps as *const fn(), &_eapps as *const fn());

    let mut ptr = start_ptr;
    while ptr < end_ptr {
        match process::Process::create(*ptr) {
            Err(_) => { return; },
            Ok(process) => {
                task::Task::Process(process).post();
            }
        }
        //task::Task::UserTask(*ptr as usize).post();
        ptr = ptr.offset(1);
    }
}

fn launch_task(task: task::Task) {
    match task {
        task::Task::UserTask(task_addr) => unsafe {
            syscall::switch_to_user(task_addr, &mut PSTACKS[0][1020] as *mut u8);
        },
        task::Task::Process(process) => unsafe {
            syscall::switch_to_user(process.pc, &mut process.memory[process.cur_stack]);
        },
        task::Task::KernelTask(task) => {
            task();
        }
    }
}

#[no_mangle]
pub extern fn main() {
    unsafe {
        task::setup();
        config::config();
        schedule_external_apps();
    }

    loop {
        match unsafe { task::dequeue() } {
            None => {
                support::wfi(); // Sleep!
            },
            Some(task) => {
                launch_task(task);
            }
        }
    }
}

