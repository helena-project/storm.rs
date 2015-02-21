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

    pub fn iterator(&mut self) -> ArrayListIterator<T> {
        ArrayListIterator{list: self, cur_index: 0}
    }

    pub fn circular_iterator(&mut self) -> CircularArrayListIterator<T> {
        CircularArrayListIterator{list: self, cur_index: 0}
    }
}

impl <T> Index<usize> for ArrayList<T> {

    type Output = T;

    fn index(&self, index: &usize) -> &T {
        let idx = *index;
        if idx >= self.len {
            panic!("Index out of bounds");
        }
        unsafe { &*self.buf.offset(idx as isize) }
    }
}

impl <T> IndexMut<usize> for ArrayList<T> {

    fn index_mut(&mut self, index: &usize) -> &mut T {
        let idx = *index;
        if idx >= self.len {
            panic!("Index out of bounds");
        }
        unsafe { &mut *self.buf.offset(idx as isize) }
    }
}

pub struct ArrayListIterator<'a, T: 'a> {
    list: &'a mut ArrayList<T>,
    cur_index: usize
}

impl<'a, T: 'a> Iterator for ArrayListIterator<'a, T> {
    type Item = &'a mut T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.len - self.cur_index;
        (remaining, Some(remaining))
    }

    fn next(&mut self) -> Option<&'a mut T> {
        unsafe {
            let idx = self.cur_index;
            if idx >= self.list.len {
                None
            } else {
                self.cur_index += 1;
                Some(&mut *self.list.buf.offset(idx as isize))
            }
        }
    }
}

pub struct CircularArrayListIterator<'a, T: 'a> {
    list: &'a mut ArrayList<T>,
    cur_index: usize
}

impl<'a, T: 'a> Iterator for CircularArrayListIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        unsafe {
            let idx = self.cur_index;
            self.cur_index = (self.cur_index + 1) % self.list.len;
            Some(&mut *self.list.buf.offset(idx as isize))
        }
    }
}

