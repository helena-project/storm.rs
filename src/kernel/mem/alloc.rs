#![cfg_attr(test, feature(core, alloc))]

#[cfg(test)]
extern crate core;

#[path="../lib/bitvector.rs"]
mod bitvector;

use core::num::{Int, FromPrimitive, UnsignedInt};
use core::mem::{size_of};
use core::ops::{Sub};
use core::raw::Slice;
use core::cmp::{min, max};
use core::slice;
use core::ptr::{self, PtrExt, MutPtrExt};
use core::intrinsics::{transmute, set_memory};

#[cfg(not(test))]
use kernel::lib::bitvector::BitVector;

#[cfg(test)]
use bitvector::BitVector;

const DEBUG: bool = true;

macro_rules! debug {
    ($($message:expr),*) => ({
        if DEBUG {
            println!($($message),*);
        }
    })
}

/*
 * This is an implementation of a binary buddy allocator in Rust. The current
 * implementation is NOT thread safe.
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
const MIN_BLOCK_SIZE: usize = 32;
const MIN_ALIGN: usize = 32;

#[repr(C, packed)]
struct Block {
    pub prev: *mut Block,
    pub next: *mut Block
}

#[repr(C, packed)]
struct ZoneMetaData {
    pub free_blocks: *mut Block,
    pub block_status: BitVector
}

pub struct BuddyAllocator {
    // Now, do we really need this? We'll see.
    offset: *mut u8,
    capacity: usize,
    allocated: usize,
    metadata: *mut [ZoneMetaData]
}

// x cannot be zero as that will result in 1 << 64 => overflow
fn pow2_rndup<T: UnsignedInt + FromPrimitive>(x: T) -> T {
    let lz = (x - Int::one()).leading_zeros() as usize;
    let bits = size_of::<T>() * 8;
    FromPrimitive::from_usize(1 << (bits - lz)).unwrap()
}

fn pow2_rnddown<T: UnsignedInt + FromPrimitive>(x: T) -> T {
    let lz = x.leading_zeros() as usize;
    let bits = size_of::<T>() * 8;
    FromPrimitive::from_usize(1 << (bits - lz - 1)).unwrap()
}

fn log2_floor<T: UnsignedInt + FromPrimitive>(x: T) -> T {
    let lz = x.leading_zeros() as usize;
    let bits = size_of::<T>() * 8;
    FromPrimitive::from_usize(bits - lz - 1).unwrap()
}

fn log2_ceil<T: UnsignedInt + FromPrimitive>(x: T) -> T {
    let lz = (x - Int::one()).leading_zeros() as usize;
    let bits = size_of::<T>() * 8;
    FromPrimitive::from_usize(bits - lz).unwrap()
}

fn round_up<T: UnsignedInt + FromPrimitive>(x: T, k: T) -> T {
     ((x + k - Int::one()) / k) * k
}

pub trait UnsizedPtrExt {
    type Target: ?Sized;

    unsafe fn as_unz_ref<'a>(&self) -> Option<&'a Self::Target>;
}

pub trait UnsizedMutPtrExt {
    type Target: ?Sized;

    unsafe fn as_unz_mut<'a>(&self) -> Option<&'a mut Self::Target>;
}

impl<P: UnsizedMutPtrExt> UnsizedPtrExt for P {
    type Target = P::Target;

    unsafe fn as_unz_ref<'a>(&self) -> Option<&'a P::Target> {
        transmute(self.as_unz_mut())
    }
}

// TODO: File bug on this.
impl<T> UnsizedMutPtrExt for *mut [T] {
    type Target = [T];

    unsafe fn as_unz_mut<'a>(&self) -> Option<&'a mut [T]> {
        let slice: Slice<T> = transmute(*self);
        if slice.data.is_null() {
            return None;
        }

        Some(transmute(*self))
    }
}

macro_rules! ptr {
    ($ptr:ident->$field:ident = $val:expr) => {
        unsafe {
            if let Some(reference) = $ptr.as_mut() {
                reference.$field = $val;
            } else {
                panic!("Cannot set null pointer!");
            }
        }
    };

    ($ptr:ident->$field:ident) => {
        unsafe {
            if let Some(reference) = $ptr.as_ref() {
                reference.$field
            } else {
                panic!("Cannot deref null pointer!");
            }
        }
    };

    ($ptr:expr) => {
        unsafe {
            if let Some(reference) = $ptr.as_unz_mut() {
                reference
            } else {
                panic!("Cannot deref null pointer!");
            }
        }
    };
}

impl BuddyAllocator {
    /*
     * Computes the size of the metadata, in bytes, needed to keep track of
     * free_mem bytes of free memory. The tracking structure is an array (slice)
     * of bit vectors, with one bit vector per block size and 1 bit being used
     * for each pair of buddies. At least one byte must be allocated per bit
     * vector.
     */
    fn metadata_size(free_mem: usize) -> usize {
        // Number of power of 2 bins we can have
        let bins = log2_floor(free_mem) - log2_floor(MIN_BLOCK_SIZE) + 1;

        // keep 1 bit per buddy pair, so at level i, i = M - k, for 2^k, need:
        // (2^k / 2) / 8 bytes for vector storage == 2^(k - 4) bytes
        // of course, need max(1, 2^(k - 4)) == 1 from [0, 4] else 2^(k - 4)
        let small_bins = min(4, bins);
        let big_bins = bins - small_bins;

        let zone_size = size_of::<ZoneMetaData>();
        let mut size = small_bins + zone_size * small_bins;
        for i in 0..big_bins {
            size += 1 << i;
            size += zone_size;
        }

        size
    }

    /*
     * Creates zone metadata at `at` using memory at `mem` for bit vector
     * storage. Assumes bits > 0.
     *
     * Returns the address (some address above `mem`) after the bit vector
     * storage.
     */
    fn mk_zone_md(at: *mut u8, mem: *mut u8, bits: usize) -> *mut u8 {
        unsafe {
            *(at as *mut ZoneMetaData) = ZoneMetaData {
                free_blocks: ptr::null_mut(),
                block_status: BitVector::from_raw(mem, bits)
            };

            let md: &mut ZoneMetaData = transmute(at);
            md.block_status.clear();
            mem.offset(((bits + 7) / 8) as isize)
        }
    }

    unsafe fn allocate_metadata(mem: *mut u8, free_mem: usize) -> (&'static mut [ZoneMetaData], usize) {
        // Number of power of 2 bins we can have
        let bins = log2_floor(free_mem) - log2_floor(MIN_BLOCK_SIZE) + 1;

        // Store slice of ZoneMetaData at `mem` and actual ZoneMetaData items at
        // `mem` + bins * sizeof(ZoneMetaData)
        let mut slice_mem = mem;
        let mut store_mem = mem.offset((bins * size_of::<ZoneMetaData>()) as isize);

        store_mem = BuddyAllocator::mk_zone_md(slice_mem, store_mem, 1);
        slice_mem = slice_mem.offset(size_of::<ZoneMetaData>() as isize);
        for i in 0..(bins - 1) {
            store_mem = BuddyAllocator::mk_zone_md(slice_mem, store_mem, 1 << i);
            slice_mem = slice_mem.offset(size_of::<ZoneMetaData>() as isize);
        }

        (slice::from_raw_parts_mut(mem as *mut ZoneMetaData, bins), store_mem as usize - mem as usize)
    }

    /*
     * @start: the address where free memory begins
     * @size: the number of bytes of free memory
     */
    pub fn new(start: *mut u8, size: usize) -> BuddyAllocator {
        let aligned_start = round_up(start as usize, MIN_ALIGN) as *mut u8;
        let free_memory = size - ((aligned_start as usize) - (start as usize));

        // TODO: Use math to precompute proper usable_free_memory without
        // retrying.
        let mut metadata_byte_len = BuddyAllocator::metadata_size(free_memory);
        let mut usable_free_memory = pow2_rnddown(free_memory);
        while free_memory - metadata_byte_len < usable_free_memory {
            usable_free_memory = pow2_rnddown(usable_free_memory - 1);
            metadata_byte_len = BuddyAllocator::metadata_size(usable_free_memory);
        }

        let (metadata, len) = unsafe {
            let metadata_start = aligned_start.offset(usable_free_memory as isize);
            BuddyAllocator::allocate_metadata(metadata_start, usable_free_memory)
        };

        if len != metadata_byte_len {
            panic!("Allocator invariant failed! {} != {} (size = {})",
                    len, metadata_byte_len, usable_free_memory);
        }

        // Setup largest block as being free
        unsafe {
            let top_block: *mut Block = transmute(aligned_start);
            set_memory::<Block>(top_block, 0, 1);
            metadata[0].free_blocks = top_block;
        }

        BuddyAllocator {
            offset: aligned_start,
            capacity: usable_free_memory,
            allocated: 0,
            metadata: metadata
        }
    }

    #[inline]
    pub fn size_to_zone(&self, size: usize) -> usize {
        let pow2_from_min = (log2_ceil(size) as isize)
            - (log2_floor(MIN_BLOCK_SIZE) as isize);
        let len = ptr!(self.metadata).len() as isize;

        if pow2_from_min >= len {
            panic!("Size {} is greater than largest zone {}!", size, len);
        }

        let zone = (len - 1) - pow2_from_min;
        if zone < 0 {
            panic!("Arithmetic error in size_to_zone!");
        }

        zone as usize
    }

    #[inline]
    pub fn zone_to_pow2(&self, zone: usize) -> usize {
        let len = ptr!(self.metadata).len() as isize;
        let zi = zone as isize;
        if zi >= len {
            panic!("Zone is out of range!");
        }

        let pow2 = (len - 1) + (log2_floor(MIN_BLOCK_SIZE) as isize - zi);
        if pow2 < 1 {
            panic!("Arithmetic error in size_to_zone!");
        }

        pow2 as usize
    }

    #[inline]
    pub fn zone_to_size(&self, zone: usize) -> usize {
        1 << self.zone_to_pow2(zone)
    }

    // TODO: Twiddle bit vector bits.
    fn remove_from_list(&mut self, md: &mut ZoneMetaData, block: *mut Block) {
        debug!("Before removal: {} free blocks.", self.list_len(md));

        if block.is_null() {
            panic!("Attempted to remove a null block from list!");
        }

        let free_blocks = md.free_blocks;
        if !free_blocks.is_null() {
            if free_blocks == block {
                md.free_blocks = ptr!(block->next);
            }
        }

        let prev = ptr!(block->prev);
        if !prev.is_null() {
            ptr!(prev->next = ptr!(block->next));
        }

        let next = ptr!(block->next);
        if !next.is_null() {
            ptr!(next->prev = ptr!(block->prev));
        }

        ptr!(block->prev = ptr::null_mut());
        ptr!(block->next = ptr::null_mut());

        debug!("After removal: {} free blocks.", self.list_len(md));
    }

    // TODO: Twiddle bit vector bits.
    fn add_to_list(&mut self, md: &mut ZoneMetaData, block: *mut Block) {
        debug!("Before add: {} free blocks.", self.list_len(md));

        if block.is_null() {
            panic!("Attempted to add a null block to list!");
        }

        let free_blocks = md.free_blocks;
        if !free_blocks.is_null() {
            ptr!(free_blocks->prev = block);
            ptr!(block->next = free_blocks);
        } else {
            ptr!(block->next = ptr::null_mut());
        }

        ptr!(block->prev = ptr::null_mut());
        md.free_blocks = block;

        debug!("After add: {} free blocks.", self.list_len(md));
    }

    fn list_len(&self, md: &mut ZoneMetaData) -> usize {
        let mut count = 0;
        let mut block = md.free_blocks;

        while !block.is_null() {
            count += 1;
            block = ptr!(block->next);
        }

        count
    }

    /**
     * Breaks up blocks beginning at `zone` so that `zone + 1` has at least two
     * free blocks in its block list after the call. Panics if the allocator
     * is out of memory.
     */
    fn break_blocks(&mut self, zone: usize) {
        debug!("Breaking blocks for zone {}", zone);
        self.stats_print();
        let mut zone_metadata = &mut ptr!(self.metadata)[zone];
        let mut free_block = zone_metadata.free_blocks;
        if free_block.is_null() {
            if zone == 0 {
                panic!("Out of memory!");
            }

            debug!("No free blocks. Moving up to {}.", zone - 1);
            self.break_blocks(zone - 1);
            debug!("Returned from zone {} breaking.", zone - 1);
            self.stats_print();

            free_block = zone_metadata.free_blocks;
        }

        if !free_block.is_null() {
            zone_metadata = &mut ptr!(self.metadata)[zone];
            self.remove_from_list(zone_metadata, free_block);
            let addr: usize = unsafe { transmute(free_block) };
            let (block1, block2) = (addr, addr + self.zone_to_size(zone + 1));

            debug!("Got 1 {} at zone {}, split to 2: {}, {} at zone {}",
                   addr, zone, block1, block2, zone + 1);

            let ref mut smalller_zone_md = ptr!(self.metadata)[zone + 1];
            unsafe {
                self.add_to_list(smalller_zone_md, transmute(block1));
                self.add_to_list(smalller_zone_md, transmute(block2));
            }
        } else {
            panic!("Failed to correctly break blocks at larger level!");
        }

        debug!("Done breaking blocks for zone {}", zone);
        self.stats_print();
    }

    fn get_free_block(&mut self, zone: usize) -> *mut Block {
        let mut zone_metadata = &mut ptr!(self.metadata)[zone];
        let free_block = zone_metadata.free_blocks;

        if free_block.is_null() && zone == 0 {
            panic!("No free block large enough to satisy request.");
        }

        if free_block.is_null() {
            debug!("No free block in zone list found. Breaking blocks.");
            self.break_blocks(zone - 1);
            let new_free_block = ptr!(self.metadata)[zone].free_blocks;
            self.remove_from_list(zone_metadata, new_free_block);
            new_free_block
        } else {
            debug!("Found free block in zone list.");
            self.remove_from_list(zone_metadata, free_block);
            free_block
        }
    }

    pub fn allocate(&mut self, size: usize, _align: usize) -> *mut u8 {
        // TODO: Use align
        debug!("-- ALLOC {} bytes", size);
        let real_size = max(MIN_BLOCK_SIZE, size_of::<Block>() + size);
        let zone = self.size_to_zone(real_size);
        debug!("-- ALLOC {} bytes (req: {} + {}) (zone {})",
            real_size, size, size_of::<Block>(), zone);

        // if there's a free block in the list, remove it and return it
        // otherwise, break up a higher level block until we have a free one
        let ref mut zone_metadata = ptr!(self.metadata)[zone];
        debug!("Zone {} has {} free blocks.", zone, self.list_len(zone_metadata));
        self.stats_print();

        let free_block = self.get_free_block(zone);
        unsafe {
            let ptr = free_block.offset(1) as *mut u8;
            debug!("Returning new block {:?}: {:?}", free_block, ptr);
            ptr
        }
    }

    // TODO: Merge buddies: needs bit vector twiddling
    pub fn deallocate(&mut self, ptr: *mut u8, old_size: usize, align: usize) {
        // TODO: Use align
        let real_size = max(MIN_BLOCK_SIZE, size_of::<Block>() + old_size);
        let zone = self.size_to_zone(real_size);
        let block_ptr = unsafe { (ptr as *mut Block).offset(-1) };

        debug!("-- DEALLOC {:?}: {:?}", ptr, block_ptr);

        let ref mut zone_metadata = ptr!(self.metadata)[zone];
        self.add_to_list(zone_metadata, block_ptr);
    }

    // pub fn reallocate(&mut self, ptr: *mut u8, old_size: usize, size: usize, align: usize)
    //         -> *mut u8 {
    //     self.offset
    // }

    pub fn stats_print(&self) {
        let num_zones = ptr!(self.metadata).len();
        debug!("==================================");
        debug!("There are {} Zones", num_zones);
        for i in 0..num_zones {
            debug!("Zone {}\t({} bytes)\t {} free blocks", i,
                self.zone_to_size(i), self.list_len(&mut ptr!(self.metadata)[i]));
        }
        debug!("==================================");
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

    use super::{BuddyAllocator, MIN_BLOCK_SIZE};
    use super::{pow2_rndup, pow2_rnddown};
    use self::alloc::heap;
    use self::core::intrinsics::{transmute, size_of, copy_memory};
    use std::default::Default;
    use std::ops::{Deref, DerefMut};
    use std::num::Int;
    use std::rand;
    use std::rand::Rng;
    use std::rand::distributions::{IndependentSample, Range};

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

    fn talloc<T: Default>(allocator: &mut BuddyAllocator, align: usize) -> RawBox<T> {
        unsafe {
            let mem = allocator.allocate(size_of::<T>(), align);
            let mut default: T = Default::default();

            // Check alignment for all allocations
            assert_eq!((mem as usize) % align, 0);

            copy_memory(mem as *mut T, &mut default as *mut T, 1);
            RawBox::new(mem as *mut T)
        }
    }

    fn tdealloc<T>(allocator: &mut BuddyAllocator, mut ptr: RawBox<T>, align: usize) {
        unsafe {
            allocator.deallocate(transmute(&mut *ptr), size_of::<T>(), align);
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

        let mut x = talloc::<u32>(&mut balloc, ALIGN);
        assert_eq!(*x, 0);

        *x = 13373;
        assert_eq!(*x, 13373);

        *x = 0;
        assert_eq!(*x, 0);

        *x = Int::max_value();
        assert_eq!(*x, Int::max_value());

        tdealloc(&mut balloc, x, ALIGN);
        unsafe { heap::deallocate(free_mem, MEM_SIZE, ALIGN); }
    }

    #[test]
    fn two_alloc_dealloc() {
        const MEM_SIZE: usize = 4096 * 4;
        let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
        let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

        let mut x = talloc::<u32>(&mut balloc, ALIGN);
        assert_eq!(*x, 0);

        let mut y = talloc::<u32>(&mut balloc, ALIGN);
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

        tdealloc(&mut balloc, y, ALIGN);
        tdealloc(&mut balloc, x, ALIGN);
        unsafe { heap::deallocate(free_mem, MEM_SIZE, ALIGN); }
    }

    // Tests that memory is defragments after deallocation.
    #[test]
    fn test_many_alloc_dealloc() {
        const ALIGN: usize = 32;
        const MEM_SIZE: usize = 4096 * 10;
        const USABLE_MEM: usize = 4096 * 8;
        let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
        let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

        // TODO: Use non deprecated RNG
        let mut between = Range::new(0, USABLE_MEM);
        let mut rng = rand::thread_rng();
        for i in 0..1000 {
            let size = between.ind_sample(&mut rng);

            let ptr = balloc.allocate(size, ALIGN);
            assert!(!ptr.is_null());

            balloc.deallocate(ptr, size, ALIGN);
        }
    }

    // Tests that blocks can be broken down fully.
    #[test]
    fn test_many_small_alloc_dealloc() {
        const ALIGN: usize = 4;
        const MEM_SIZE: usize = 4096 * 10;
        const USABLE_MEM: usize = 4096 * 8;
        let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
        let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

        let mut allocs: Vec<*mut u8> = vec![];
        for i in 0..(USABLE_MEM / MIN_BLOCK_SIZE) {
            let ptr = balloc.allocate(1, ALIGN);
            assert!(!ptr.is_null());
            allocs.push(ptr);
        }

        for ptr in allocs {
            balloc.deallocate(ptr, 1, ALIGN);
        }
    }

    // Ensures that an OOM error occurs after too many allocations.
    #[test]
    #[should_fail]
    fn test_too_many_small_alloc_dealloc() {
        const ALIGN: usize = 4;
        const MEM_SIZE: usize = 4096 * 10;
        const USABLE_MEM: usize = 4096 * 8;
        let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
        let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

        let mut allocs: Vec<*mut u8> = vec![];
        for i in 0..(USABLE_MEM / MIN_BLOCK_SIZE + 1) {
            let ptr = balloc.allocate(1, ALIGN);
            assert!(!ptr.is_null());
            allocs.push(ptr);
        }

        for ptr in allocs {
            balloc.deallocate(ptr, 1, ALIGN);
        }
    }

    // #[test]
    // fn test_align() {
    //     const MEM_SIZE: usize = 4096 * 4;
    //     let free_mem = unsafe { heap::allocate(MEM_SIZE, ALIGN) };
    //     let mut balloc = BuddyAllocator::new(free_mem, MEM_SIZE);

    //     for i in 1..12 {
    //         // talloc will check alignment
    //         let mut x = talloc::<u32>(&mut balloc, 1 << i);
    //         assert_eq!(*x, 0);
    //         *x = 13373;
    //         assert_eq!(*x, 13373);
    //         tdealloc(&mut balloc, x, 1 << i);
    //     }

    //     unsafe { heap::deallocate(free_mem, MEM_SIZE, ALIGN); }
    // }
}

