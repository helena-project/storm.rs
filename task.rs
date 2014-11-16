use core::option::{Option, None, Some};
use intrinsics;

struct Task(pub fn());

struct TaskManager {
    head: uint,
    tail: uint,
    tasks: [Option<Task>, ..100]
}

static mut MANAGER : TaskManager = TaskManager { head: 0, tail: 0, tasks: [None,..100] };

impl Task {
    pub fn post(&self) -> bool {
        unsafe { MANAGER.enqueue(*self) }
    }
}

impl TaskManager {
    pub fn enqueue(&mut self, task: Task) -> bool {
            loop {
                let tail = self.tail;
                let next_tail = (tail + 1) % 100;

                // Do not continue if we may overrung the head of the task
                // buffer.
                if next_tail == self.head {
                    return false;  
                }

                unsafe {
                    if next_tail != intrinsics::atomic_cxchg(&mut self.tail,
                                                             tail, next_tail) {
                      continue;  
                    }
                }
                self.tasks[next_tail] = Some(task);
                break;
            }
        return true;
    }

    pub fn dequeue(&self) -> Option<Task> {
        return None;
    }
}
