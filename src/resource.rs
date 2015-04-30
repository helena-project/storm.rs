#[cfg(test)] extern crate core;

use core::prelude::*;
use core::cell::UnsafeCell;

pub struct Resource<T> {
    value: UnsafeCell<T>
}

impl<T> Resource<T> {
    pub fn new(value: T) -> Resource<T> {
        Resource {
            value: UnsafeCell::new(value)
        }
    }

    pub fn with<F,R>(&self, mut f: F) -> R
            where F: FnMut(&mut T) -> R {
        let me = unsafe { &mut *self.value.get() };
        f(me)
    }

    pub unsafe fn borrow_mut(&self) -> &mut T {
        &mut *self.value.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = &Resource::new(1234);

        let f1 = || {
            r.with(|r| {
                *r + 1
            })
        };

        let f2 = || {
            r.with(|r| {
                *r + 1
            })
        };

        assert_eq!(1235, f1());
    }

    static mut i : isize = 0;

    struct Foo;

    impl Drop for Foo {
        fn drop(&mut self) {
            unsafe {
                i += 1;
            }
        }
    }


    #[test]
    fn test_drop() {
        let cur_i = unsafe { i };
        {
            let r = &Resource::new(Foo);
        }
        assert_eq!(cur_i + 1, unsafe { i });
    }
}

