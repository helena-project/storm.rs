use core::prelude::*;
use core::intrinsics::atomic_xadd;

/// Size of each processes's memory region in bytes
pub const PROC_MEMORY_SIZE : usize = 2048;

static mut MEMORIES: [[u8; PROC_MEMORY_SIZE]; 8] = [[0; PROC_MEMORY_SIZE]; 8];
static mut FREE_MEMORY_IDX: usize = 0;

static mut CURRENT_PROCESS : Option<Process> = None;

pub struct Process {
    /// The process's memory.
    pub memory: &'static mut [u8; PROC_MEMORY_SIZE],

    /// The offset in `memory` to use for the process stack.
    pub cur_stack: usize,

    /// The next instruction to invoke when returning to the process.
    pub pc: usize
}

pub unsafe fn swap_current_process(process: Option<Process>) -> Option<Process> {
    let res = CURRENT_PROCESS.take();
    CURRENT_PROCESS = process;
    return res;
}

impl Process {
    pub fn create(init_fn: fn()) -> Result<Process, ()> {
        unsafe {
            let cur_idx = atomic_xadd(&mut FREE_MEMORY_IDX, 1);
            if cur_idx > MEMORIES.len() {
                atomic_xadd(&mut FREE_MEMORY_IDX, -1);
                Err(())
            } else {
                Ok(Process {
                    memory: &mut MEMORIES[cur_idx],
                    cur_stack: PROC_MEMORY_SIZE - 4,
                    pc: init_fn as usize
                })
            }
        }
    }
}

