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
struct BuddyAllocator {
    // Now, do we really need this? We'll see.
    offset: usize,

    capacity: usize,
    allocated: usize
}

impl BuddyAllocator {
    /*
     * @start: the address where free memory begins
     * @size: the number of bytes of free memory
     */
    fn new(start: *const usize, size: usize) -> BuddyAllocator {
        BuddyAllocator {
            offset: start as usize,
            capacity: size,
            allocated: 0
        }
    }
}
