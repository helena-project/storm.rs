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
        let next_tail = (self.tail + 1) % self.buf.len();

        // Do not continue if we may overrun the head of the element
        // buffer.
        if next_tail == self.head {
            return false;
        }

        self.buf[self.tail] = Some(elm);
        self.tail = next_tail;
        return true;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        let elm = self.buf[self.head].take();
        match elm {
            None => None,
            result@Some(_) => {
                self.head = (self.head + 1) % self.buf.len();
                result
            }
        }
    }

    pub fn peek(&self) -> &Option<T> {
        &self.buf[self.head]
    }
}

