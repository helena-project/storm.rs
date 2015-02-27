use core::prelude::*;

pub trait Timer {
    fn now(&self) -> u32;
    fn set_alarm(&mut self, u32);
    fn disable_alarm(&mut self);
}

pub trait AlarmHandler {
    fn fire_alarm<F: FnMut(*mut (), usize, usize, usize, usize)>(&mut self, F);
}
