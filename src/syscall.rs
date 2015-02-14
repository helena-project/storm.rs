#[allow(improper_ctypes)]
extern {
    fn __ctx_to_user(start: usize, user_stack: *mut u8) -> u16;
    fn __ctx_to_master();
}

fn noop(_: usize, _: usize) -> isize { -1 }

pub static mut SUBSCRIBE_DRIVERS: [fn(usize, usize) -> isize; 10] = [noop; 10];
pub static mut NUM_SUBSCRIBE_DRIVERS: usize = 0;

pub static mut CMD_DRIVERS: [fn(usize, usize) -> isize; 10] = [noop; 10];
pub static mut NUM_CMD_DRIVERS: usize = 0;

enum Syscalls {
    WAIT = 0,
    SUBSCRIBE = 1,
    COMMAND = 2
}

pub const WAIT: u16 = 0;
pub const SUBSCRIBE: u16 = 1;
pub const COMMAND: u16 = 2;

pub unsafe fn switch_to_user(pc: usize, sp: *mut u8) -> u16 {
    __ctx_to_user(pc, sp)
    //let icsr: *mut usize = 0xE000ED04 as *mut usize;
    //volatile_store(icsr, volatile_load(icsr as *const usize) | 1<<28);
}

#[derive(Copy)]
pub enum ReturnTo {
  Process = 0,
  Kernel = 1
}

/*#[no_mangle]
#[allow(unused_assignments)]
/// Called from the SVC handler.
pub unsafe extern fn svc_rust_handler(psp: usize) -> u16 {
    use core::intrinsics::volatile_load;

    /* Find process PC on stack */
    let user_pc = volatile_load((psp + 24) as *const usize);
    let r0 = volatile_load(psp as *const usize);
    let r1 = volatile_load((psp + 4) as *const usize);
    let r2 = volatile_load((psp + 8) as *const usize);

    /* SVC is one instruction before current PC. The low byte is the opcode */
    let svc = volatile_load((user_pc - 2) as *const u16) & 0xff;

    return svc;
    /*match svc {
        WAIT => {
            volatile_store(psp as *mut isize, 0);
        },
        SUBSCRIBE => {
            if r0 > NUM_SUBSCRIBE_DRIVERS {
                volatile_store(psp as *mut isize, -1);
            }

            let res = SUBSCRIBE_DRIVERS[r0](r1, r2);
            volatile_store(psp as *mut isize, res);
            return ReturnTo::Process;
        },
        COMMAND => {
            if r0 > NUM_CMD_DRIVERS {
                volatile_store(psp as *mut isize, -1);
            }

            let res = CMD_DRIVERS[r0](r1, r2);
            volatile_store(psp as *mut isize, res);
            return ReturnTo::Process;
        },
        _ => {
            volatile_store(psp as *mut isize, -1);
        }
    }
    return ReturnTo::Kernel;*/
}
*/
