// use core::prelude;

// pub enum Mode { // Mode is encoded as CPOL in bit 0 and NCPHA in bit 1
//     Mode0 = 2,  // CPOL == 0, NCPHA = 1
//     Mode1 = 0,  // CPOL == 0, NCPHA = 0
//     Mode2 = 3,  // CPOL == 1, NCPHA = 1
//     Mode3 = 1   // CPOL == 1, NCPHA = 0
// }

// impl prelude::Copy for Mode {}





pub trait I2C {
	fn enable (&mut self);
	fn disable (&mut self);


    /// Write a slice of bytes to a particular slave.
    /// This call is synchronous and will block until all bytes have written
    fn write_sync (&mut self, data: &[u8]);

    // Issue a read transaction to fill the buffer slice with data.
    // This call is synchronous and will block until all bits have been read.
    fn read_sync (&mut self, buffer: &mut[u8]);
}
