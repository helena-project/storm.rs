#[cfg(test)]
extern crate core;

use core::num::{Int, FromPrimitive, UnsignedInt};
use core::mem;
use core::ops::{Sub};

/*
 * This is an implementation of a binary buddy allocator in Rust.
 *
 * @author Sergio Benitez <sbenitez@stanford.edu>
 *
 * A binary buddy allocator begins with a chunk of memory of size 2^L where L is
 * floor(log_2(free memory amount)), in other words, the total amount of free
 * memory rounded down to the nearest power of two. The allocator keeps lists
 * of free memory of sizes 2^k for k from 2 to L.
 *
 * A request to allocate memory of size M is rounded up the nearest power of
 * two: 2^m, where m = ceil(log_2(M)). The request is satisfied as follows:
 *   1) The first non-empty list of blocks in ranges 2^m to 2^L is found.
 *     - let k be this first exponent so that the free block is 2^k.
 *     - if no such block is found, there is no suitable free memory
 *   2) If k == m, then the block is simply returned.
 *   3) If k != m, then:
 *     a) The block of size 2^k is split into two blocks of sizes 2^(k-1)
 *     b) The 2^k-sized block is removed from its respective free-list.
 *     c) The 2^(k-1) sized blocks are added to their respective free list.
 *     d) k is set to k - 1; step 2 is repeated with a 2^(k-1) block
 *
 * The allocator gets it name from splitting blocks into two leaving each of
 * the split blocks with a buddy - the other half of the split block. Computing
 * the buddy of a block is simple: because blocks are sized as power of twos,
 * say, 2^k, the buddy of a block can be found by flipping the bit in the kth
 * position.
 *
 * Consider the following example where each |----| is a block and the numbers
 * at each | correspond to the start/end addresses.
 *
 * 0                                                                         2^8
 * |--------------------------------------------------------------------------|
 * |------------------------------------|-------------------------------------|
 * 0                                   2^7                                  2^8
 * |-----------------|--------E---------|
 * 0                2^6               2^7
 * |---A----|----B---|---C----|---D-----|
 * 0        2^5     2^6   (2^5 + 2^6)  2^7
 *
 * Let's say we want to merge blocks C and D back into E.
 *   Their size is:              2^5 = 0b0100000
 *   C's start address is:       2^6 = 0b1000000
 *   D's start address is: 2^5 + 2^6 = 0b1100000
 *
 * By flipping C's bit 5, we get D's start address, and by flipping D's bit 5,
 * we get C's start address.
 *
 * A request to free memory at address A of size 2^m is satisfied as follows:
 *   0) If m == L, then A is added to the free list for 2^L; return
 *   1) The buddy is calculated: B = (A xor (all 1s with zero at position m))
 *   2) If the buddy is in its respective free list:
 *      a) The buddy is removed from its free list.
 *      b) A block at min(A, B) with size 2^(m + 1) is added to its free list.
 *      c) A is set to min(A, B) and m is set to m + 1; step 0 is repeated
 *   3) If the buddy has not been freed, A is added to its free list.
 */
pub struct BuddyAllocator {
    // Now, do we really need this? We'll see.
    offset: *mut u8,

    capacity: usize,
    allocated: usize
}

// x cannot be zero as that will result in 1 << 64 => overflow
fn pow2_rndup<T: UnsignedInt + FromPrimitive>(x: T) -> T {
    let lz = (x - Int::one()).leading_zeros();
    let bits = mem::size_of::<T>() * 8;
    FromPrimitive::from_uint(1 << (bits - lz)).unwrap()
}

fn pow2_rnddown<T: UnsignedInt + FromPrimitive>(x: T) -> T {
    let lz = x.leading_zeros();
    let bits = mem::size_of::<T>() * 8;
    FromPrimitive::from_uint(1 << (bits - lz - 1)).unwrap()
}

impl BuddyAllocator {
    /*
     * @start: the address where free memory begins
     * @size: the number of bytes of free memory
     */
    pub fn new(start: *mut u8, size: usize) -> BuddyAllocator {
        BuddyAllocator {
            offset: start,
            capacity: size,
            allocated: 0
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        self.offset
    }

    pub fn deallocate(&mut self, ptr: *mut u8, old_size: usize, align: usize) {

    }

    pub fn reallocate(&mut self, ptr: *mut u8, old_size: usize, size: usize, align: usize)
            -> *mut u8 {
        self.offset
    }

    pub fn stats_print(&self) {

    }
}

#[cfg(test)]
mod tests {
    /*
     * We test the buddy allocator using the host OS allocator as a source of
     * free memory (heap::allocate, heap::deallocate).
     */
    extern crate alloc;
    extern crate core;

    use super::BuddyAllocator;
    use super::{pow2_rndup, pow2_rnddown};
    use self::alloc::heap;
    use self::core::intrinsics;
    use std::default::Default;
    use std::ops::{Deref, DerefMut};
    use std::num::Int;

    const ALIGN: usize = 4;

    #[allow(raw_pointer_derive)]
    #[derive(Debug)]
    struct RawBox<T> {
        ptr: *mut T
    }

    impl<T> RawBox<T> {
        fn new(ptr: *mut T) -> RawBox<T> {
            RawBox {
                ptr: ptr
            }
        }
    }

    impl<T> Deref for RawBox<T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe { &*(self.ptr) }
        }
    }

    impl<T> DerefMut for RawBox<T> {
        fn deref_mut(&mut self) -> &mut T {
            unsafe { &mut *(self.ptr) }
        }
    }

    fn talloc<T: Default>(allocator: &mut BuddyAllocator) -> RawBox<T> {
        unsafe {
            let mem = allocator.allocate(intrinsics::size_of::<T>(), ALIGN);
            let mut default: T = Default::default();

            intrinsics::copy_memory(mem as *mut T, &mut default as *mut T, 1);
            RawBox::new(mem as *mut T)
        }
    }

    #[test]
    fn test_rndup_pow_2() {
        assert_eq!(pow2_rndup(0b1000u32), 0b1000u32);
        assert_eq!(pow2_rndup(0b1001u32), 0b10000u32);
        assert_eq!(pow2_rndup(0b1000u64), 0b1000u64);
        assert_eq!(pow2_rndup(0b1001u64), 0b10000u64);

        assert_eq!(pow2_rndup(0b1u8), 0b1u8);
        assert_eq!(pow2_rndup(0b1u16), 0b1u16);
        assert_eq!(pow2_rndup(0b1u32), 0b1u32);
        assert_eq!(pow2_rndup(0b1u64), 0b1u64);
        assert_eq!(pow2_rndup(0b11u32), 0b100u32);
        assert_eq!(pow2_rndup(0b11u64), 0b100u64);
        assert_eq!(pow2_rndup(!(1u64 << 63)), 1 << 63);

        assert_eq!(pow2_rndup((1u32 << 20) | 1 << 19), 1 << 21);
        assert_eq!(pow2_rndup((1u32 << 30) | 1 << 29), 1 << 31);
        assert_eq!(pow2_rndup((1u64 << 62) | 1 << 32), 1 << 63);
    }

    #[test]
    fn test_rnddown_pow_2() {
        assert_eq!(pow2_rnddown(0b1000u8), 0b1000u8);
        assert_eq!(pow2_rnddown(0b1000u32), 0b1000u32);
        assert_eq!(pow2_rnddown(0b1001u32), 0b1000u32);
        assert_eq!(pow2_rnddown(0b1001u64), 0b1000u64);

        assert_eq!(pow2_rnddown(0b1u8), 0b1u8);
        assert_eq!(pow2_rnddown(0b1u16), 0b1u16);
        assert_eq!(pow2_rnddown(0b1u32), 0b1u32);
        assert_eq!(pow2_rnddown(0b1u64), 0b1u64);
        assert_eq!(pow2_rnddown(0b11u32), 0b10u32);
        assert_eq!(pow2_rnddown(0b11u64), 0b10u64);
        assert_eq!(pow2_rnddown(!(1u64 << 63)), 1 << 62);

        assert_eq!(pow2_rnddown(-1u64), 1u64 << 63);
        assert_eq!(pow2_rnddown(-500u64), 1u64 << 63);
    }

    #[test]
    fn simple_alloc_dealloc() {
        const MEM_SIZE: usize = 4096 * 4;
        let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
        let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

        let mut x = talloc::<u32>(&mut balloc);
        assert_eq!(*x, 0);

        *x = 13373;
        assert_eq!(*x, 13373);

        *x = 0;
        assert_eq!(*x, 0);

        *x = Int::max_value();
        assert_eq!(*x, Int::max_value());

        unsafe { heap::deallocate(free_mem, MEM_SIZE, ALIGN); }
    }

    #[test]
    fn two_alloc_dealloc() {
        const MEM_SIZE: usize = 4096 * 4;
        let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
        let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

        let mut x = talloc::<u32>(&mut balloc);
        assert_eq!(*x, 0);

        let mut y = talloc::<u32>(&mut balloc);
        assert_eq!(*y, 0);

        *x = 13373;
        assert_eq!(*x, 13373);
        assert_eq!(*y, 0);

        *y = 221122;
        assert_eq!(*y, 221122);
        assert_eq!(*x, 13373);

        *x = Int::max_value();
        assert_eq!(*x, Int::max_value());
        assert_eq!(*y, 221122);

        *y = *x;
        assert_eq!(*y, Int::max_value());
        assert_eq!(*x, *y);

        unsafe { heap::deallocate(free_mem, MEM_SIZE, ALIGN); }
    }
}

