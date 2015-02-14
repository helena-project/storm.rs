use core::prelude::*;
use core::ops::{Index,IndexMut};

pub struct ArrayList<T: Sized> {
    pub len: usize,
    pub cap: usize,
    pub buf: *mut T
}

impl <T> ArrayList<T> {
    pub unsafe fn new(cap: usize, buf: *mut T) -> ArrayList<T> {
        ArrayList{ len: 0, cap: cap, buf: buf}
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn add(&mut self, elm: T) -> bool {
        if self.cap <= self.len {
            return false
        }
        unsafe {
            *self.buf.offset(self.len as isize) = elm;
        }
        self.len += 1;
        return true;
    }
}

impl <T> Index<usize> for ArrayList<T> {
    type Output = T;

    fn index(&self, index: &usize) -> &T {
        let idx = index % self.len;
        unsafe { &*self.buf.offset(idx as isize) }
    }
}

impl <T> IndexMut<usize> for ArrayList<T> {
    type Output = T;

    fn index_mut(&mut self, index: &usize) -> &mut T {
        let idx = index % self.len;
        unsafe { &mut *self.buf.offset(idx as isize) }
    }
}

