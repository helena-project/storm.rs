use core::intrinsics::*;
use task;
use timer;

#[allow(improper_ctypes)]
extern {
    fn __prepare_user_stack(start : uint, user_stack : *mut uint);
    fn __ctx_to_user();
    fn __ctx_to_master();
}

pub const YIELD : u16 = 0;
pub const ADD_TIMER : u16 = 1;

pub unsafe fn switch_to_user(pc: uint, sp: *mut uint) {
    __prepare_user_stack(pc, sp);
    let icsr : *mut uint = 0xE000ED04 as *mut uint;
    volatile_store(icsr, volatile_load(icsr as *const uint) | 1<<28);
}

#[no_mangle]
#[allow(non_snake_case)]
#[allow(unused_assignments)]
pub unsafe extern fn SVC_Handler(r0 : uint, r1 : uint) {
    use core::intrinsics::volatile_load;

    let mut psp : uint = 0;
    asm!("mrs $0, PSP" :"=r"(psp)::: "volatile");

    /* Find process PC on stack */
    let user_pc = volatile_load((psp + 24) as *const uint);

    /* SVC is one instruction before current PC. The low byte is the opcode */
    let svc = volatile_load((user_pc - 2) as *const u16) & 0xff;
    match svc {
        YIELD => {},
        ADD_TIMER => {
            let alarm_task = task::Task::UserTask(r1);
            timer::set_alarm(r0 as u32, alarm_task);
            return ();
        },
        _ => {}
    }

    __ctx_to_master();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn PendSV_Handler() {
    __ctx_to_user();
}
