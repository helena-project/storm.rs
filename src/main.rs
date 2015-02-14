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

#[inline(never)]
fn launch_task(task: task::Task) -> u16 {
    match task {
        task::Task::UserTask(task_addr) => unsafe {
            syscall::switch_to_user(task_addr, &mut PSTACKS[0][1020] as *mut u8)
        },
        task::Task::Process(process) => unsafe {
            syscall::switch_to_user(process.pc,
                &mut process.memory[process.cur_stack] as *mut u8)
        }
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

    //let subscribe_drivers = unsafe { syscall::SUBSCRIBE_DRIVERS };
    //let cmd_drivers = unsafe { syscall::CMD_DRIVERS };

    let mut console = unsafe { config::Console.as_mut().unwrap() };

    console.putc(proc_list.len() as u8 + 48);
    console.writeln(" procs");
    loop {
        for i in range(0, proc_list.len()) {
            console.write("proc ");
            console.putc(i as u8 + 48);
            console.writeln("");
            let process = &mut proc_list[i];
            let svc = unsafe {
                syscall::switch_to_user(process.pc,
                    &mut process.memory[process.cur_stack])
            };
            match svc {
                syscall::WAIT => {
                    unsafe { config::Console.as_mut().unwrap().writeln("wait") };
                },
                syscall::SUBSCRIBE => {
                    unsafe { config::Console.as_mut().unwrap().writeln("subscribe") };
                },
                syscall::COMMAND => {
                    unsafe { config::Console.as_mut().unwrap().writeln("command") };
                },
                _ => {
                    unsafe { config::Console.as_mut().unwrap().writeln("unrecognized") };
                }
            };
        }
    }
        /*match unsafe { task::dequeue() } {
            None => {
                support::wfi(); // Sleep!
            },
            Some(task) => {
                let svc = launch_task(task);
                let out = match svc {
                    syscall::WAIT => "wait",
                    syscall::SUBSCRIBE => "subscribe",
                    syscall::COMMAND => "command",
                    _ => "unrecognized"
                };
                unsafe {
                    config::Console.as_mut().expect("").writeln(out);
                }
            }
        }*/
}

