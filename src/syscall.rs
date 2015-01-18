use core::intrinsics::*;

#[allow(improper_ctypes)]
extern {
    fn __prepare_user_stack(start : usize, user_stack : *mut usize);
    fn __ctx_to_user();
    fn __ctx_to_master();
}

fn noop(_ : usize, _ : usize) -> isize { -1 }

pub static mut SUBSCRIBE_DRIVERS : [fn(usize, usize) -> isize;10] = [noop;10];
pub static mut NUM_SUBSCRIBE_DRIVERS : usize = 0;

pub static mut CMD_DRIVERS : [fn(usize, usize) -> isize;10] = [noop;10];
pub static mut NUM_CMD_DRIVERS : usize = 0;

pub const WAIT : u16 = 0;
pub const SUBSCRIBE : u16 = 1;
pub const COMMAND : u16 = 2;

pub unsafe fn switch_to_user(pc: usize, sp: *mut usize) {
    __prepare_user_stack(pc, sp);
    let icsr : *mut usize = 0xE000ED04 as *mut usize;
    volatile_store(icsr, volatile_load(icsr as *const usize) | 1<<28);
}

#[no_mangle]
#[allow(non_snake_case)]
#[allow(unused_assignments)]
pub unsafe extern fn SVC_Handler() {
    use core::intrinsics::volatile_load;

    let mut psp : usize = 0;
    asm!("mrs $0, PSP" :"=r"(psp)::: "volatile");

    /* Find process PC on stack */
    let user_pc = volatile_load((psp + 24) as *const usize);

    /* SVC is one instruction before current PC. The low byte is the opcode */
    let svc = volatile_load((user_pc - 2) as *const u16) & 0xff;
    match svc {
        WAIT => {
            volatile_store(psp as *mut isize, 0);
        },
        SUBSCRIBE => {
            let r0 = volatile_load((psp) as *const usize);
            if r0 > NUM_SUBSCRIBE_DRIVERS {
                volatile_store(psp as *mut isize, -1);
            }
            let r1 = volatile_load((psp + 4) as *const usize);
            let r2 = volatile_load((psp + 8) as *const usize);

            let res : isize = SUBSCRIBE_DRIVERS[r0](r1, r2);
            volatile_store(psp as *mut isize, res);
            return;
        },
        COMMAND => {
            let r0 = volatile_load((psp) as *const usize);
            if r0 > NUM_CMD_DRIVERS {
                volatile_store(psp as *mut isize, -1);
            }
            let r1 = volatile_load((psp + 4) as *const usize);
            let r2 = volatile_load((psp + 8) as *const usize);

            let res : isize = CMD_DRIVERS[r0](r1, r2);
            volatile_store(psp as *mut isize, res);
            return;
        },
        _ => {
            volatile_store(psp as *mut isize, -1);
        }
    }
    __ctx_to_master();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn PendSV_Handler() {
    __ctx_to_user();
}

