use core::mem;
use core::prelude::*;
use core::ptr;
use core::ops::{Deref,DerefMut};

pub struct Ptr<T>(ptr::Unique<T>);

// TODO(alevy): Implement drop/deallocationg. Probably requires an explicit
// deallocator stored in the struct
impl<T> Ptr<T> {
    pub fn new(mut value: T, dst: &mut [u8]) -> Option<Ptr<T>> {
        let val_size = mem::size_of_val(&value);
        if val_size == dst.len() {
            unsafe {
                let dst_mem = mem::transmute(&mut dst[0]);
                ptr::copy_memory(&mut value, dst_mem, val_size);
                Some(mem::transmute(dst_mem))
            }
        } else {
            None
        }
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &**self
    }
}

impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut T { &mut **self }
}

pub struct Node<T> {
    next: Link<T>,
    prev: *mut Node<T>,
    value: T
}

type Link<T> = Option<Ptr<Node<T>>>;

pub struct LinkedList<T> {
    list_head: Link<T>,
    list_tail: *mut Node<T>
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList{list_head: None, list_tail: ptr::null_mut()}
    }

    pub fn front(&self) -> Option<&T> {
        self.list_head.as_ref().map(|head| &head.value)
    }

    pub fn pop_front_node(&mut self) -> Option<Ptr<Node<T>>> {
        self.list_head.take().map(|mut front_node| {
            match front_node.next.take() {
                None => self.list_tail = ptr::null_mut(),
                Some(mut node) => {
                    node.prev = ptr::null_mut();
                    self.list_head = Some(node);
                }
            }
            front_node
        })
    }

    pub fn push_front_node(&mut self, mut new_head: Ptr<Node<T>>) {
        match self.list_head {
            None => {
                new_head.prev = ptr::null_mut();
                self.list_tail = &mut *new_head;
                self.list_head = Some(new_head);
            },
            Some(ref mut head) => {
                new_head.prev = ptr::null_mut();
                head.prev = &mut *new_head;
                mem::swap(head, &mut new_head);
                head.next = Some(new_head);
            }
        }
    }

    pub fn push_front(&mut self, elm: T) {
        let node = Ptr::new(Node {
            next: None,
            prev: ptr::null_mut(),
            value: elm
        }, &mut [0,1,2,3,4,5,5,6]).unwrap();

        self.push_front_node(node)
    }

    pub fn iter(&self) -> LinkedListIterator<T> {
        LinkedListIterator {head: &self.list_head}
    }
    
    pub fn iter_mut(&mut self) -> LinkedListIterMut<T> {
        let p = match self.list_head {
            None => ptr::null_mut(),
            Some(ref mut head) => &mut **head as *mut Node<T>
        };
        LinkedListIterMut {head: p}
    }
}

impl<T: Copy> LinkedList<T> {
    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(|node| { node.value })
    }
}

pub struct LinkedListIterator<'a, T: 'a> {
    head: &'a Link<T>
}

impl<'a, T> Iterator for LinkedListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.head.as_ref().map(|head| {
            self.head = &head.next;
            &head.value
        })
    }
}

pub struct LinkedListIterMut<T> {
    // A raw pointer because we want to bypass ownership semantics
    head: *mut Node<T>
}

impl<'a, T> Iterator for LinkedListIterMut<T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let hd = unsafe { self.head.as_mut() };
        hd.map(|head| {
            self.head = match head.next {
                None => ptr::null_mut(),
                Some(ref mut node) => {
                    &mut **node
                }
            };
            &mut head.value
        })
    }
}

