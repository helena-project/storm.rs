use core::option::Option;
use core::option::Option::*;
use hal::ast;
use hil::timer::Timer;

use task;
use task::Task;
use ringbuf::RingBuf;

#[deriving(Copy)]
pub struct Alarm {
    pub task: Task,
    pub tics: u32
}

const MAX_ALARMS: uint = 100;

static mut ALARM_BUF: [Option<Alarm>,..MAX_ALARMS] = [None,..MAX_ALARMS];

pub static mut ALARMS: RingBuf<Alarm> =
  RingBuf { head: 0
          , tail: 0
          , cap: 0
          , buf: 0 as *mut Option<Alarm>
          };

pub fn set_user_alarm(tics: uint, task_addr: uint) -> int {
    unsafe {
        let task = Task::UserTask(task_addr);
        let tics = tics as u32;
        let cur_time = ast::Ast0.get_counter();
        let alarm = Alarm { task: task, tics: tics + cur_time};
        ALARMS.enqueue(alarm);

        if ALARMS.len() == 1 {
          ast::Ast0.disable();
          ast::Ast0.clear_alarm();
          ast::Ast0.enable_alarm_irq();
          ast::Ast0.set_alarm(alarm.tics);
          ast::Ast0.set_alarm_callback(ast_alarm_handler);
          ast::Ast0.enable();
        }
        return 0;
    }
}

pub fn setup() {
    unsafe {
        ALARMS.buf = &mut ALARM_BUF[0] as *mut Option<Alarm>;
        ALARMS.cap = MAX_ALARMS;
        ast::Ast0.setup();
    }
}

fn handle_alarm() {
    unsafe {
        match ALARMS.dequeue() {
            None => (),
            Some(cur_alarm) => {
                cur_alarm.task.post();
                match ALARMS.peek() {
                    None => (),
                    Some(alarm) => {
                        ast::Ast0.enable_alarm_irq();
                        ast::Ast0.set_alarm(alarm.tics);
                        ast::Ast0.enable();
                    }
                }
            }
        }

    }
}

fn ast_alarm_handler() {
    unsafe {
        task::Task::KernelTask(handle_alarm).post();
        ast::Ast0.disable();
        ast::Ast0.clear_alarm();
    }
}

