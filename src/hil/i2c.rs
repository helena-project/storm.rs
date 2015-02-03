// use core::prelude;

// pub enum Mode { // Mode is encoded as CPOL in bit 0 and NCPHA in bit 1
//     Mode0 = 2,  // CPOL == 0, NCPHA = 1
//     Mode1 = 0,  // CPOL == 0, NCPHA = 0
//     Mode2 = 3,  // CPOL == 1, NCPHA = 1
//     Mode3 = 1   // CPOL == 1, NCPHA = 0
// }

// impl prelude::Copy for Mode {}





pub trait I2CSlaveFns {
    /// Write a slice of bytes to a particular slave (given by its address).
    /// This call is synchronous and will block until all bytes have written
    fn write_sync (&self, data: &[u8]);

    // Read count bytes from a slave.
    // This call is synchronous and will block until all bits have been read.
    // TODO(alevy): commented out just to get a compiling version of I2C. This
    // _may_ be a problematic type since it means allocating the slice
    // somewhere. Perhaps passing in a buffer would be better?
    //fn read_sync (&self, count: usize) -> &[u8];
}
