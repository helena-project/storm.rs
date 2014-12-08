use core::option::*;
use ast;
use task;
use task::Task;
use ringbuf::RingBuf;

pub struct Alarm {
    pub task : Task,
    pub tics : u32
}

const MAX_ALARMS : uint = 100;

static mut ALARM_BUF : [Option<Alarm>,..MAX_ALARMS] = [None,..MAX_ALARMS];

pub static mut ALARMS : RingBuf<Alarm> =
  RingBuf { head: 0
          , tail: 0
          , cap: 0
          , buf: 0 as *mut Option<Alarm>
          };

pub fn set_alarm(tics : u32, task : fn()) {
    let cur_time = ast::get_counter();
    let alarm = Alarm { task: Task(task), tics: tics + cur_time};
    unsafe {
        ALARMS.enqueue(alarm);
    }

    if unsafe { ALARMS.len() } == 1 {
      ast::disable();
      ast::clear_alarm();
      ast::enable_alarm_irq();
      ast::set_alarm(alarm.tics);
      ast::enable();
    }

}

pub fn setup() {
    unsafe {
        ALARMS.buf = &mut ALARM_BUF[0] as *mut Option<Alarm>;
        ALARMS.cap = MAX_ALARMS;
    }
    ast::select_clock(ast::Clock::ClockRCSys);
    ast::set_prescalar(0);
    ast::clear_alarm();
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
                        ast::enable_alarm_irq();
                        ast::set_alarm(alarm.tics);
                        ast::enable();
                    }
                }
            }
        }

    }
}

pub fn ast_alarm_handler() {
    task::post(handle_alarm);
    ast::disable();
    ast::clear_alarm();
}

