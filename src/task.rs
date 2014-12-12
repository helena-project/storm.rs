use core::option::{Option, None};
use ringbuf::RingBuf;

pub struct Task(pub fn());

const MAX_TASKS : uint = 10;
static mut TASK_BUF : [Option<Task>,..MAX_TASKS] = [None,..MAX_TASKS];

pub static mut MANAGER : RingBuf<Task> =
    RingBuf { head: 0, tail: 0, cap: MAX_TASKS,
              buf: 0 as *mut Option<Task> };

pub unsafe fn setup() {
    MANAGER.buf = &mut TASK_BUF[0] as *mut Option<Task>;
}

pub unsafe fn dequeue() -> Option<Task> {
    MANAGER.dequeue()
}

pub fn post(func: fn()) -> bool {
    Task(func).post()
}

impl Task {
    pub fn post(self) -> bool {
        unsafe { MANAGER.enqueue(self) }
    }
}

