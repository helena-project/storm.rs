#[cfg(test)] extern crate core;

use core::prelude::*;
use core::cell::UnsafeCell;

pub struct Shared<T: Sync> {
    value: UnsafeCell<T>
}

impl<T: Sync> Shared<T> {
    pub fn new(value: T) -> Shared<T> {
        Shared {
            value: UnsafeCell::new(value)
        }
    }

    pub fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }
}

