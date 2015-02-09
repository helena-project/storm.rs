use core::prelude::*;
use ringbuf::RingBuf;
use process;

pub enum Task {
    Process(process::Process),
    UserTask(usize),
    KernelTask(fn())
}

const MAX_TASKS: usize = 10;
static mut TASK_BUF: [Option<Task>; MAX_TASKS] = [None, None, None, None, None, None, None, None, None, None];

pub static mut MANAGER: RingBuf<Task> =
    RingBuf { head: 0, tail: 0, cap: MAX_TASKS,
              buf: 0 as *mut Option<Task> };

pub unsafe fn setup() {
    MANAGER.buf = &mut TASK_BUF[0] as *mut Option<Task>;
}

pub unsafe fn dequeue() -> Option<Task> {
    MANAGER.dequeue()
}

impl Task {
    pub fn post(self) -> bool {
        unsafe { MANAGER.enqueue(self) }
    }
}

