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

fn do_syscall(svc_num: u16, r0: usize, r1: usize, _: usize) {
    let cmd_drivers = unsafe { syscall::CMD_DRIVERS };
    match svc_num {
        syscall::WAIT => {
            //unsafe { config::Console.as_mut().unwrap().writeln("wait") };
        },
        syscall::SUBSCRIBE => {
            //unsafe { config::Console.as_mut().unwrap().writeln("subscribe") };
        },
        syscall::COMMAND => {
            cmd_drivers[r0](r1, 0);
        },
        _ => {
            //unsafe { config::Console.as_mut().unwrap().writeln("unrecognized") };
        }
    };
}

unsafe fn svc_and_registers(psp: *const usize) -> (u16, usize, usize, usize) {
    use core::intrinsics::volatile_load;

    let pcptr = volatile_load((psp as *const *const u16).offset(6));
    let svc_instr = volatile_load(pcptr.offset(-1));
    let r0 = volatile_load(psp);
    let r1 = volatile_load(psp.offset(1));
    let r2 = volatile_load(psp.offset(2));
    (svc_instr & 0xff, r0, r1, r2)
}

#[no_mangle]
pub extern fn main() {
    use core::prelude::*;

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

    loop {
        for i in range(0, proc_list.len()) {
            let process = &mut proc_list[i];
            let psp = unsafe {
                syscall::switch_to_user(process.pc,
                    &mut process.memory[process.cur_stack])
            };
            let (svc_num, r0, r1, r2) = unsafe { svc_and_registers(psp) };
            do_syscall(svc_num, r0, r1, r2);
        }
    }
}

