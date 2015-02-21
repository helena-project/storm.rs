use core::prelude::*;

pub struct RingBuffer<'a, T: 'a> {
    pub head: usize,
    pub tail: usize,
    pub buf: &'a mut [Option<T>]
}

impl <'a, T> RingBuffer<'a, T> {
    pub fn new(buf: &'a mut [Option<T>]) -> RingBuffer<'a, T> {
        RingBuffer {
            head: 0, tail: 0, buf: buf
        }
    }

    pub fn len(&self) -> usize {
        (self.tail + self.buf.len() - self.head) % self.buf.len()
    }

    pub fn enqueue(&mut self, elm: T) -> bool {
        unsafe {
            asm!("mov r4, 1; msr PRIMASK, r4" :::"r4": "volatile");
        }
        let next_tail = (self.tail + 1) % self.buf.len();

        let result =
            if next_tail == self.head {
                // Do not continue if we may overrun the head of the element
                // buffer.
                false
            } else {
                self.buf[self.tail] = Some(elm);
                self.tail = next_tail;
                true
            };
        unsafe {
            asm!("mov r4, 0; msr PRIMASK, r4" :::"r4": "volatile");
        }
        return result;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        unsafe {
            asm!("mov r4, 1; msr PRIMASK, r4" :::"r4": "volatile");
        }
        let elm = self.buf[self.head].take();
        let res = match elm {
            None => None,
            result@Some(_) => {
                self.head = (self.head + 1) % self.buf.len();
                result
            }
        };
        unsafe {
            asm!("mov r4, 0; msr PRIMASK, r4" :::"r4": "volatile");
        }
        return res;
    }

    pub fn peek(&self) -> &Option<T> {
        &self.buf[self.head]
    }
}

