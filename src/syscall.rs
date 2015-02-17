#[allow(improper_ctypes)]
extern {
    pub fn switch_to_user(user_stack: *mut u8) -> *mut u8;
}

pub type SyscallFunc = fn(usize, usize) -> isize;

fn noop(_: usize, _: usize) -> isize { -1 }

pub static mut SUBSCRIBE_DRIVERS: [SyscallFunc; 10] = [noop; 10];
pub static mut NUM_SUBSCRIBE_DRIVERS: usize = 0;

pub static mut CMD_DRIVERS: [SyscallFunc; 10] = [noop; 10];
pub static mut NUM_CMD_DRIVERS: usize = 0;

pub const WAIT: u8 = 0;
pub const SUBSCRIBE: u8 = 1;
pub const COMMAND: u8 = 2;

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
