/*
 * I2C Support for the Atmel SAM4L
 *
 */

// extern crate core;


use core::prelude::SliceExt;
use core::intrinsics;

use hil::i2c;



// Listing of all registers related to the TWIM peripheral.
// Section 27.9 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct I2CRegisters {
    control:                         usize,
    clock_waveform_generator:        usize,
    smbus_timing:                    usize,
    command:                         usize,
    next_command:                    usize,
    receive_holding:                 usize,
    transmit_holding:                usize,
    status:                          usize,
    interrupt_enable:                usize,
    interrupt_disable:               usize,
    interrupt_mask:                  usize,
    status_clear:                    usize,
    parameter:                       usize,
    version:                         usize,
    hsmode_clock_waveform_generator: usize,
    slew_rate:                       usize,
    hsmod_slew_rate:                 usize
}

// This maps to "TWIMS0" in section 7.1
// bradjc: I have no idea if this is the right one.
pub const I2C_BASE: usize = 0x40018000;

// GI2C points to the I2C registers in memory
static mut GI2C: *mut I2CRegisters = I2C_BASE as *mut I2CRegisters;

// bradjc: I don't understand why this can't go in the hil file. That would
// seem to make a lot more sense.
#[allow(missing_copy_implementations)]
pub struct I2CSlave {
    pub address: u16
}


/// This enables the entire I2C peripheral
pub fn enable () {
    unsafe {
        intrinsics::volatile_store(&mut(*GI2C).control, 0x00000001);
    }
}

/// This disables the entire I2C peripheral
pub fn disable () {
    unsafe {
        intrinsics::volatile_store(&mut(*GI2C).control, 0x00000002);
    }
}

/// Call this to initialize the I2C hardware. This should likely be done
/// once by the platform to set the parameters of the bus. Because I2C slaves
/// must all read the same bus there should really be just one configuration.
pub fn init () {

}


impl i2c::I2CSlaveFns for I2CSlave {
    fn write_sync (&self, data: &[u8]) {
        unsafe {
            let command = (data.len() << 16) |  // NBYTES
                          (0x1 << 15) |         // VALID
                          (0x1 << 13) |         // START
                          (0x0 << 11) |         // TENBIT
                          ((self.address as usize) << 1) | // SADR
                          (0x0 << 0);           // READ

            intrinsics::volatile_store(&mut(*GI2C).command, command);

            // Write all bytes in the data buffer to the I2C peripheral
            for i in 0..data.len() {
                // Wait for the peripheral to tell us that we can
                // write to the TX register
                loop {
                    let status = intrinsics::volatile_load(&(*GI2C).status);
                    if status & 0x00000002 == 0x00000002 {
                        break;
                    }
                }
                intrinsics::volatile_store(&mut(*GI2C).transmit_holding, data[i] as usize);
            }
        }
    }

    /*fn read_sync (&self, count: usize) -> &[u8] {

    }*/
}


