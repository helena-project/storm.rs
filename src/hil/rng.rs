/*
 * Interface for reading random numbers.
 */

pub trait RNG {
    /// Read a 32 bit random number. This will block until the number is ready.
    fn read_sync (&mut self) -> u32;

    /// Read multiple 32 bit random numbers.
    /// Pass in a slice that is long enough to hold the resulting random
    /// numbers.
    fn read_multiple_sync (&mut self, count: usize, vals: &mut[u32]);
}
