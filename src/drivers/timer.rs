use core::prelude::*;
use collections::linked_list::LinkedList;
use hil::timer::{AlarmHandler, Timer};

#[derive(Copy)]
struct Alarm {
    armed: bool,
    origin: u32,
    duration: u32,
    cb_ptr: *mut (),
    cb_addr: usize
}

pub struct VirtualTimer<T: Timer> {
    timer: T,
    active: bool,
    alarms: LinkedList<Alarm>
}

impl <T: Timer> AlarmHandler for VirtualTimer<T> {
    fn fire_alarm<F>(&mut self, mut post: F) where
            F: FnMut(*mut (), usize, usize, usize, usize) {
        self.timer.disable_alarm();
        for cur in self.alarms.iter_mut() {
            if cur.armed {
                cur.armed = false;
                post(cur.cb_ptr, cur.cb_addr, 0, 0, 0);
            }
        }
    }
}

impl <T: Timer> VirtualTimer<T> {
    pub fn initialize(timer: T) -> VirtualTimer<T> {
        VirtualTimer {timer: timer, active: false, alarms: LinkedList::new()}
    }

    pub fn set_user_alarm(&mut self, cb_ptr: *mut (), duration: u32, cb: usize)
            -> isize {
        let now = self.timer.now();
        let alarm = Alarm { armed: true,
                            origin: now,
                            duration: duration,
                            cb_ptr: cb_ptr,
                            cb_addr: cb
                          };
        if !self.add_alarm(alarm) {
            return -1;
        }
        if !self.active {
            let mut min_remaining = alarm.duration + alarm.origin - now;
            for cur in self.alarms.iter() {
                let elapsed = now - cur.origin;
                let remaining = cur.duration - elapsed;
                if remaining < min_remaining {
                    min_remaining = remaining;
                }
            }
            let next_alarm = now + min_remaining;
            self.timer.set_alarm(next_alarm);
        }
        return 0;
    }

    fn add_alarm(&mut self, alarm: Alarm) -> bool {
        self.alarms.push_front(alarm);
        return true;
    }
}

