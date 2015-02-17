use core::prelude::*;
use core::intrinsics::{atomic_xadd, volatile_load, volatile_store};

use syscall;

/// Size of each processes's memory region in bytes
pub const PROC_MEMORY_SIZE : usize = 2048;

static mut MEMORIES: [[u8; PROC_MEMORY_SIZE]; 8] = [[0; PROC_MEMORY_SIZE]; 8];
static mut FREE_MEMORY_IDX: usize = 0;

pub struct Process {
    /// The process's memory.
    pub memory: &'static mut [u8; PROC_MEMORY_SIZE],

    /// The offset in `memory` to use for the process stack.
    pub cur_stack: *mut u8,
}

impl Process {
    pub fn create(init_fn: fn()) -> Result<Process, ()> {
        unsafe {
            let cur_idx = atomic_xadd(&mut FREE_MEMORY_IDX, 1);
            if cur_idx > MEMORIES.len() {
                atomic_xadd(&mut FREE_MEMORY_IDX, -1);
                Err(())
            } else {
                let memory = &mut MEMORIES[cur_idx];

                // Fill in initial stack expected by SVC handler
                // Top minus 8 u32s for r0-r3, r12, lr, pc and xPSR
                let stack_bottom : *mut usize =
                    &mut memory[PROC_MEMORY_SIZE - 32] as *mut u8 as *mut usize;
                volatile_store(stack_bottom.offset(7), 0x01000000);
                volatile_store(stack_bottom.offset(6), init_fn as usize);
                Ok(Process {
                    memory: &mut MEMORIES[cur_idx],
                    cur_stack: stack_bottom as *mut u8
                })
            }
        }
    }

    /// Context switch to the process.
    pub unsafe fn switch_to(&mut self) {
        let psp = syscall::switch_to_user(self.cur_stack);
        self.cur_stack = psp;
    }

    pub fn svc_number(&self) -> Option<u8> {
        let psp = self.cur_stack as *const *const u16;
        unsafe {
            let pcptr = volatile_load((psp as *const *const u16).offset(6));
            let svc_instr = volatile_load(pcptr.offset(-1));
            Some((svc_instr & 0xff) as u8)
        }
    }

    pub fn r0(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr) }
    }

    pub fn set_r0(&mut self, val: isize) {
        let pspr = self.cur_stack as *mut isize;
        unsafe { volatile_store(pspr, val) }
    }

    pub fn r1(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr.offset(1)) }
    }

    pub fn r2(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr.offset(2)) }
    }

}

