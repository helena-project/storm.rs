use core::prelude::*;
use hal;
use hil::timer::AlarmHandler;
use drivers;
use syscall;
use task::Task::UserTask;

pub static mut VirtualTimer :
    Option<drivers::timer::VirtualTimer<hal::ast::Ast>> = None;

pub fn virtual_timer_driver_callback() {
    unsafe {
        let mut vt = VirtualTimer.take().unwrap();
        vt.fire_alarm(|&: addr| {
            UserTask(addr).post();
        });
        VirtualTimer = Some(vt);
    }
}

pub fn virtual_timer_driver_svc(r1 : usize, r2 : usize) -> isize {
    unsafe {
        let mut vt = VirtualTimer.take().unwrap();
        let res = vt.set_user_alarm(r1 as u32, r2);
        VirtualTimer = Some(vt);
        return res;
    }
}

pub unsafe fn config() {
    let mut ast = hal::ast::Ast::new(virtual_timer_driver_callback);
    ast.setup();
    VirtualTimer = Some(drivers::timer::VirtualTimer::initialize(ast));
    syscall::DRIVERS[0] = virtual_timer_driver_svc;
    syscall::NUM_DRIVERS = 1;
}

