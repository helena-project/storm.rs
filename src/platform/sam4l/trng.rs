/*
 * TRNG Support for the Atmel SAM4L.
 */

// use core::prelude::SliceExt;
use core::intrinsics;

use hil;
use sam4l;

// Section 35.4 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct TRNGRegisters {
    control:              usize,
    reserved1:           [usize; 3],
    interrupt_enable:     usize,
    interrupt_disable:    usize,
    interrupt_mask:       usize,
    interrupt_status:     usize,
    reserved2:           [usize; 12],
    output_data:          usize,
    reserved3:           [usize; 42],
    version:              usize
}

// The addresses in memory (7.1 of manual) of the TRNG peripheral
const TRNG_BASE_ADDR: usize = 0x40068000;

// Only one TRNG
// #[derive(Copy)]
pub enum TRNGLocation {
    TRNG
}

// These parameters are passed in from the platform's device tree.
pub struct TRNGParams {
    pub location: TRNGLocation
}

pub struct TRNGDevice {
    registers: &'static mut TRNGRegisters,  // Pointer to the TRNG registers in memory
    clock: sam4l::pm::Clock
}

// Need to implement the `new` function on the TRNG device as a constructor.
// This gets called from the device tree.
impl TRNGDevice {
    pub fn new (params: TRNGParams) -> TRNGDevice {
        // return
        TRNGDevice {
            registers: unsafe { intrinsics::transmute(TRNG_BASE_ADDR) },
            clock: sam4l::pm::Clock::PBA(sam4l::pm::PBAClock::TRNG)
        }
    }

    fn enable (&mut self) {
        // Enable the clock for the TRNG module
        sam4l::pm::enable_clock(self.clock);

        // Enable. Need to write the magic number 0x524E47 to the register
        // in order for any write to work.
        volatile!(self.registers.control = (0x524E47 << 8) | 0x1);
    }

    fn disable (&mut self) {
        // Disable. Need to write the magic number 0x524E47 to the register
        // in order for any write to work.
        volatile!(self.registers.control = 0x524E47 << 8);
        sam4l::pm::disable_clock(self.clock);
    }
}


impl hil::rng::RNG for TRNGDevice {

    fn read_sync (&mut self) -> u32 {
        self.enable();

        // Loop until the random number
        loop {
            let status = volatile!(self.registers.interrupt_status);
            // DATRDY
            if status & 0x1 == 0x1 {
                break;
            }
        }
        let out = volatile!(self.registers.output_data);

        self.disable();

        // return
        out as u32
    }

    fn read_multiple_sync (&mut self, count: usize, vals: &mut[u32]) {
        self.enable();

        for i in 0..count {
            loop {
                let status = volatile!(self.registers.interrupt_status);
                // DATRDY
                if status & 0x1 == 0x1 {
                    break;
                }
            }
            vals[i] = volatile!(self.registers.output_data) as u32;
        }

        self.disable();
    }
}
