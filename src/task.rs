use core::option::{Option, None, Some};

pub struct Task(pub fn());

const MAX_TASKS : uint = 10;

pub struct TaskManager {
    pub head: uint,
    pub tail: uint,
    pub tasks: [Option<Task>, ..MAX_TASKS]
}

pub static mut MANAGER : TaskManager =
  TaskManager { head: 0, tail: 0, tasks: [None,..MAX_TASKS] };

impl Task {
    pub fn post(self) -> bool {
        unsafe { MANAGER.enqueue(self) }
    }
}

impl TaskManager {
    pub fn enqueue(&mut self, task: Task) -> bool {
        let next_tail = (self.tail + 1) % MAX_TASKS;

        // Do not continue if we may overrung the head of the task
        // buffer.
        if next_tail == self.head {
            return false;
        }
        self.tasks[self.tail] = Some(task);
        self.tail = next_tail;
        return true;
    }

    pub unsafe fn dequeue(&mut self) -> Option<Task> {
        match self.tasks[self.head] {
            None => None,
            result@Some(_) => {
                self.tasks[self.head] = None;
                self.head = (self.head + 1) % MAX_TASKS;
                result
            }
        }
    }
}

pub unsafe fn dequeue() -> Option<Task> {
    MANAGER.dequeue()
}

pub fn post(func: fn()) -> bool {
    Task(func).post()
}

