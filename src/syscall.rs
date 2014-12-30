use core::intrinsics::*;

#[allow(improper_ctypes)]
extern {
    fn __prepare_user_stack(start : uint, user_stack : *mut uint);
    fn __ctx_to_user();
    fn __ctx_to_master();
}

fn noop(_ : uint, _ : uint) -> int { -1 }

pub static mut DRIVERS : [fn(uint, uint) -> int,..10] = [noop,..10];
pub static mut NUM_DRIVERS : uint = 0;

pub const WAIT : u16 = 0;
pub const SUBSCRIBE : u16 = 1;
pub const COMMAND : u16 = 2;

pub unsafe fn switch_to_user(pc: uint, sp: *mut uint) {
    __prepare_user_stack(pc, sp);
    let icsr : *mut uint = 0xE000ED04 as *mut uint;
    volatile_store(icsr, volatile_load(icsr as *const uint) | 1<<28);
}

#[no_mangle]
#[allow(non_snake_case)]
#[allow(unused_assignments)]
pub unsafe extern fn SVC_Handler() {
    use core::intrinsics::volatile_load;

    let mut psp : uint = 0;
    asm!("mrs $0, PSP" :"=r"(psp)::: "volatile");

    /* Find process PC on stack */
    let user_pc = volatile_load((psp + 24) as *const uint);

    /* SVC is one instruction before current PC. The low byte is the opcode */
    let svc = volatile_load((user_pc - 2) as *const u16) & 0xff;
    match svc {
        WAIT => {
            volatile_store(psp as *mut int, 0);
        },
        SUBSCRIBE => {
            let r0 = volatile_load((psp) as *const uint);
            if r0 > NUM_DRIVERS {
                volatile_store(psp as *mut int, -1);
            }
            let r1 = volatile_load((psp + 4) as *const uint);
            let r2 = volatile_load((psp + 8) as *const uint);

            let res : int = DRIVERS[r0](r1, r2);
            volatile_store(psp as *mut int, res);
            return;
        },
        COMMAND => {
            volatile_store(psp as *mut int, -1);
        },
        _ => {
            volatile_store(psp as *mut int, -1);
        }
    }
    __ctx_to_master();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn PendSV_Handler() {
    __ctx_to_user();
}

