#[cfg(not(test))]
use core::prelude::*;

// ALWAYS! change both of these at the same time (run the tests, too!)
const BITS_PER_SLOT: usize = 8;
pub type BitStore = *mut u8;

#[repr(C, packed)]
pub struct BitVector {
    storage: BitStore,
    num_bits: usize
}

impl BitVector {
    pub unsafe fn from_raw(storage: BitStore, bit_len: usize) -> BitVector {
        BitVector {
            storage: storage,
            num_bits: bit_len
        }
    }

    #[inline]
    fn index_to_indexes(i: usize) -> (usize, usize) {
        (i / BITS_PER_SLOT, i % BITS_PER_SLOT)
    }

    pub fn get(&self, i: usize) -> Option<bool> {
        if i >= self.num_bits { return None }
        let (slot, bit) = BitVector::index_to_indexes(i);

        unsafe {
            Some((*self.storage.offset(slot as isize) & (1 << bit)) > 0)
        }
    }

    pub fn set(&mut self, i: usize, val: bool) {
        if i >= self.num_bits { panic!("OOB!") }
        let (slot, bit) = BitVector::index_to_indexes(i);
        unsafe {
            if val {
                *self.storage.offset(slot as isize) |= 1 << bit;
            } else {
                *self.storage.offset(slot as isize) &= !(1 << bit);
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            for i in 0..((self.num_bits + BITS_PER_SLOT - 1) / BITS_PER_SLOT) {
                *self.storage.offset(i as isize) = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BitVector;
    use super::BITS_PER_SLOT;
    use std::cmp::{min, max};
    use std::raw::{self, Repr};

    macro_rules! do_for_ranges {
        ($param:ident in $($range:expr),+ do $action:block) => {{
            $(for $param in $range $action)+
        }};
    }

    fn vec_for_bits(num_bits: usize) -> Vec<u8> {
        let num_slots = (num_bits + BITS_PER_SLOT - 1) / BITS_PER_SLOT;
        let mut storage = Vec::with_capacity(num_slots);
        unsafe { storage.set_len(num_slots) }

        for slot in storage.iter_mut() {
            *slot = 0;
        }

        storage
    }

    fn vec2ptr(v: &mut Vec<u8>) -> *mut u8 {
        let repr: raw::Slice<u8> = v.as_mut_slice().repr();
        repr.data as *mut u8
    }

    #[test]
    fn test_1_element() {
        const NUM_BITS: usize = 100;

        for i in 0..NUM_BITS {
            let mut storage = vec_for_bits(NUM_BITS);
            let ptr = vec2ptr(&mut storage);
            let mut bitv = unsafe { BitVector::from_raw(ptr, NUM_BITS) };

            bitv.set(i, true);
            assert!(bitv.get(i).unwrap());
            assert_eq!(bitv.get(i), Some(true));

            do_for_ranges!(j in 0..i, (i + 1)..(NUM_BITS) do {
                assert!(!bitv.get(j).unwrap(),
                    "i: {}; bitv[{}] is {:?}", i, j, bitv.get(j));
                assert_eq!(bitv.get(j), Some(false));
            });
        }
    }

    #[test]
    #[should_fail]
    fn test_simple_failure() {
        const NUM_BITS: usize = 10;
        let mut storage = vec_for_bits(NUM_BITS);
        let ptr = vec2ptr(&mut storage);
        let mut bitv = unsafe { BitVector::from_raw(ptr, NUM_BITS) };
        bitv.set(NUM_BITS, false);
    }

    #[test]
    #[should_fail]
    fn test_simple_failure2() {
        const NUM_BITS: usize = 10;
        let mut storage = vec_for_bits(NUM_BITS);
        let ptr = vec2ptr(&mut storage);
        let bitv = unsafe { BitVector::from_raw(ptr, NUM_BITS) };
        bitv.get(NUM_BITS).unwrap();
    }

    #[test]
    fn test_2_elements() {
        const NUM_BITS: usize = 93;

        for i in 0..NUM_BITS {
            for j in 0..NUM_BITS {
                let mut storage = vec_for_bits(NUM_BITS);
                let ptr = vec2ptr(&mut storage);
                let mut bitv = unsafe { BitVector::from_raw(ptr, NUM_BITS) };

                bitv.set(i, true);
                bitv.set(j, true);
                assert!(bitv.get(i).unwrap());
                assert!(bitv.get(j).unwrap());
                assert_eq!(bitv.get(i), Some(true));
                assert_eq!(bitv.get(j), Some(true));

                let (l, r) = (min(i, j), max(i, j));
                if l == r {
                    do_for_ranges!(k in 0..l, (l + 1)..(NUM_BITS) do {
                        assert!(!bitv.get(k).unwrap(),
                            "l: {}; bitv[{}] is {:?}",
                            l, k, bitv.get(k));
                        assert_eq!(bitv.get(k), Some(false));
                    });
                } else {
                    do_for_ranges!(k in 0..l, (l + 1)..(r), (r + 1)..NUM_BITS do {
                        assert!(!bitv.get(k).unwrap(),
                            "l, r: ({}, {}); bitv[{}] is {:?}",
                            l, r, k, bitv.get(k));
                        assert_eq!(bitv.get(k), Some(false));
                    });
                }
            }
        }
    }

    #[test]
    fn test_all_elements_and_clear() {
        const NUM_BITS: usize = 1037;
        let mut storage = vec_for_bits(NUM_BITS);
        let ptr = vec2ptr(&mut storage);
        let mut bitv = unsafe { BitVector::from_raw(ptr, NUM_BITS) };

        for i in 0..NUM_BITS {
            assert!(!bitv.get(i).unwrap());
            assert_eq!(bitv.get(i), Some(false));
        }

        for i in 0..NUM_BITS {
            bitv.set(i, true);
        }

        for i in 0..NUM_BITS {
            assert!(bitv.get(i).unwrap());
            assert_eq!(bitv.get(i), Some(true));
        }
        bitv.clear();
        for i in 0..NUM_BITS {
            assert!(!bitv.get(i).unwrap());
            assert_eq!(bitv.get(i), Some(false));
        }
    }

    #[test]
    fn test_bad_get() {
        const NUM_BITS: usize = 100;
        let mut storage = vec_for_bits(NUM_BITS);
        let ptr = vec2ptr(&mut storage);
        let bitv = unsafe { BitVector::from_raw(ptr, NUM_BITS) };
        assert_eq!(bitv.get(101), None);
        assert_eq!(bitv.get(102), None);
        assert_eq!(bitv.get(-1), None);
    }
}

