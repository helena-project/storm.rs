use core::prelude::*;
use hil::timer::{AlarmHandler, Timer};

struct Alarm {
    armed: bool,
    origin: u32,
    duration: u32,
    cb_addr: usize
}

impl Copy for Alarm {}

pub struct VirtualTimer<T: Timer> {
    timer: T,
    active: bool,
    alarms: [Alarm; 10],
}

impl <T: Timer> AlarmHandler for VirtualTimer<T> {
    fn fire_alarm<F: FnMut(usize)>(&mut self, mut post: F) {
        //let now = self.timer.now();
        self.timer.disable_alarm();
        for i in range(0, 10) {
            let cur = &mut self.alarms[i];
            if cur.armed {
                cur.armed = false;
                post(cur.cb_addr);
            }
            /*let remaining = cur.duration + cur.origin - now;
            if remaining <= 0 {
                cur.armed = false;
                post(cur.cb_addr);
            }*/
        }
    }
}

impl <T: Timer> VirtualTimer<T> {
    pub fn initialize(timer: T) -> VirtualTimer<T> {
        let base_alarm = Alarm {
            armed: false, 
            origin: 0, 
            duration: 0, 
            cb_addr: 0
        };
        VirtualTimer {timer: timer, active: false, alarms: [base_alarm; 10]}
    }

    pub fn set_user_alarm(&mut self, duration: u32, cb: usize) -> isize {
        let now = self.timer.now();
        let alarm = Alarm { armed: true,
                            origin: now,
                            duration: duration,
                            cb_addr: cb
                          };
        if !self.add_alarm(alarm) {
            return -1;
        }
        if !self.active {
            let mut min_remaining = alarm.duration + alarm.origin - now;
            for i in range(0, 10) {
                let cur = self.alarms[i];
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
        for i in range(0, 10) {
            if !self.alarms[i].armed {
                self.alarms[i] = alarm;
                return true;
            }
        }
        return false;
    }
}

