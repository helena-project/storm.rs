/*
 * Read the CHIPID from the Atmel SAM4L.
 */

use core::intrinsics;

// Section 9.3 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct CHIPIDRegisters {
    chip_id:           usize,
    chip_id_extension: usize
}

// The addresses in memory (7.1 of manual)
const CHIPID_BASE_ADDR: usize = 0x400E0740;

// Only one
// #[derive(Copy)]
pub enum CHIPIDLocation {
    CHIPID
}

// These parameters are passed in from the platform's device tree.
pub struct CHIPIDParams {
    pub location: CHIPIDLocation
}

pub struct CHIPIDDevice {
    registers: &'static mut CHIPIDRegisters
}

// Need to implement the `new` function on the TRNG device as a constructor.
// This gets called from the device tree.
impl CHIPIDDevice {
    pub fn new (params: CHIPIDParams) -> CHIPIDDevice {
        // return
        CHIPIDDevice {
            registers: unsafe { intrinsics::transmute(CHIPID_BASE_ADDR) }
        }
    }

    pub fn read (&mut self) -> (u8, u8, u8, u8, u8, u8, u8, u8, bool, bool, bool, bool, u8) {
        let cidr = volatile!(self.registers.chip_id);
        let cidre = volatile!(self.registers.chip_id_extension);

        // return
        (
        (cidr & 0x1F)         as u8,   // VERSION
        ((cidr >> 5) & 0x7)   as u8,   // EPROC
        ((cidr >> 8) & 0xF)   as u8,   // NVPSIZ
        ((cidr >> 12) & 0xF)  as u8,   // NCPSIZ2
        ((cidr >> 16) & 0xF)  as u8,   // SRAMSIZ
        ((cidr >> 20) & 0xFF) as u8,   // ARCH
        ((cidr >> 28) & 0x7)  as u8,   // NVPTYP
        ((cidr >> 31) & 0x1)  as u8,   // EXT
        ((cidre >> 0) & 0x1)  == 1,    // AES
        ((cidre >> 1) & 0x1)  == 1,    // USB
        ((cidre >> 2) & 0x1)  == 1,    // USBFULL
        ((cidre >> 3) & 0x1)  == 1,    // LCD
        ((cidre >> 24) & 0x7) as u8,   // PACKAGE
        )
    }
}
