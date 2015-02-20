/*
 * I2C Support for the Atmel SAM4L.
 *
 * Uses the TWIM peripheral.
 */

use core::prelude::SliceExt;
use core::intrinsics;

use hil;
use sam4l;



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

// The addresses in memory (7.1 of manual) of the TWIM peripherals
const I2C_BASE_ADDRS: [usize; 4] = [0x40018000, 0x4001C000, 0x40078000, 0x4007C000];

// There are four TWIM (two wire master interface) peripherals on the SAM4L.
// These likely won't all be used for I2C, but we let the platform decide
// which one to use.
#[derive(Copy)]
pub enum I2CLocation {
    I2CPeripheral00,  // TWIMS0
    I2CPeripheral01,  // TWIMS1
    I2CPeripheral02,  // TWIM2
    I2CPeripheral03   // TWIM3
}

// Three main I2C speeds
#[derive(Copy)]
pub enum I2CSpeed {
    Standard100k,
    Fast400k,
    FastPlus1M
}

// These parameters are passed in from the platform's device tree.
#[derive(Copy)]
pub struct I2CParams {
    pub location: I2CLocation,
    pub bus_speed: I2CSpeed
}

// This is instantiated when an I2C device is created by the device tree.
// This represents an abstraction of the peripheral hardware.
pub struct I2CDevice {
    registers: &'static mut I2CRegisters,  // Pointer to the I2C registers in memory
    clock: sam4l::pm::Clock
}

// Need to implement the `new` function on the I2C device as a constructor.
// This gets called from the device tree.
impl I2CDevice {
    pub fn new (params: I2CParams) -> I2CDevice {
        let address: usize = I2C_BASE_ADDRS[params.location as usize];

        // Create the actual device
        let mut device = I2CDevice {
            registers: unsafe { intrinsics::transmute(address) },
            clock: match params.location {
                I2CLocation::I2CPeripheral00 => sam4l::pm::Clock::PBA(sam4l::pm::PBAClock::TWIM0),
                I2CLocation::I2CPeripheral01 => sam4l::pm::Clock::PBA(sam4l::pm::PBAClock::TWIM1),
                I2CLocation::I2CPeripheral02 => sam4l::pm::Clock::PBA(sam4l::pm::PBAClock::TWIM2),
                I2CLocation::I2CPeripheral03 => sam4l::pm::Clock::PBA(sam4l::pm::PBAClock::TWIM3)
            }
        };

        sam4l::pm::enable_clock(device.clock);

        // enable, reset, disable
        volatile!(device.registers.control = 0x1 << 0);
        volatile!(device.registers.control = 0x1 << 7);
        volatile!(device.registers.control = 0x1 << 1);

        // Init the bus speed
        device.set_bus_speed(params.bus_speed);

        // slew
        volatile!(device.registers.slew_rate = (0x2 << 28) | (7<<16) | (7<<0));

        // clear interrupts
        volatile!(device.registers.status_clear = 0xFFFFFFFF);

        // return
        device
    }

    /// Set the clock prescaler and the time widths of the I2C signals
    /// in the CWGR register to make the bus run at a particular I2C speed.
    pub fn set_bus_speed (&mut self, bus_speed: I2CSpeed) {

        // Set the clock speed parameters. This could be made smarter, but for
        // now we just use precomputed constants based on a 48MHz clock.
        // See line 320 in asf-2.31.0/sam/drivers/twim/twim.c for where I
        // got these values.
        // clock_speed / bus_speed / 2
        let (exp, data, stasto, high, low) = match bus_speed {
            I2CSpeed::Standard100k => (0, 0, 240, 240, 240),
            I2CSpeed::Fast400k =>     (0, 0,  60,  60,  60),
            I2CSpeed::FastPlus1M =>   (0, 0,  24,  24,  24)
            // I2CSpeed::Standard100k => (7, 10, 200, 100, 100),
            // I2CSpeed::Fast400k =>     (7, 10, 200, 100, 100),
            // I2CSpeed::FastPlus1M =>   (7, 10, 200, 100, 100)
        };

        let cwgr = ((exp & 0x7) << 28) |
                   ((data & 0xF) << 24) |
                   ((stasto & 0xFF) << 16) |
                   ((high & 0xFF) << 8) |
                   ((low & 0xFF) << 0);
        volatile!(self.registers.clock_waveform_generator = cwgr);
    }
}


impl hil::i2c::I2C for I2CDevice {

    /// This enables the entire I2C peripheral
    fn enable (&mut self) {
        volatile!(self.registers.control = 0x00000001);
    }

    /// This disables the entire I2C peripheral
    fn disable (&mut self) {
        volatile!(self.registers.control = 0x00000002);
        sam4l::pm::disable_clock(self.clock);
    }

    fn write_sync (&mut self, addr: u16, data: &[u8]) {

        // enable, reset, disable
        volatile!(self.registers.control = 0x1 << 0);
        volatile!(self.registers.control = 0x1 << 7);
        volatile!(self.registers.control = 0x1 << 1);


        // Configure the command register to instruct the TWIM peripheral
        // to execute the I2C transaction
        let command = (data.len() << 16) |             // NBYTES
                      (0x1 << 15) |                    // VALID
                      (0x1 << 14) |                    // STOP
                      (0x1 << 13) |                    // START
                      (0x0 << 11) |                    // TENBIT
                      ((addr as usize) << 1) |         // SADR
                      (0x0 << 0);                      // READ
        volatile!(self.registers.command = command);

        volatile!(self.registers.control = 0x1 << 0);

        // Write all bytes in the data buffer to the I2C peripheral
        for i in 0..data.len() {
            // Wait for the peripheral to tell us that we can
            // write to the TX register
            loop {
                let status = volatile!(self.registers.status);
                if status & 0x00000002 == 0x00000002 {
                    break;
                }
            }
            volatile!(self.registers.transmit_holding = data[i] as usize);
        }
    }

    fn read_sync (&mut self, addr: u16, buffer: &mut[u8]) {

        // enable, reset, disable
        volatile!(self.registers.control = buffer[0] as usize);
        volatile!(self.registers.control = 0x1 << 7);
        volatile!(self.registers.control = 0x1 << 1);


        // Configure the command register to instruct the TWIM peripheral
        // to execute the I2C transaction
        let command = (buffer.len() << 16) |           // NBYTES
                      (0x1 << 15) |                    // VALID
                      (0x1 << 13) |                    // START
                      (0x0 << 11) |                    // TENBIT
                      ((addr as usize) << 1) |         // SADR
                      (0x1 << 0);                      // READ
        volatile!(self.registers.command = command);

        volatile!(self.registers.control = 0x1 << 0);

        // Read bytes in to the buffer
        for i in 0..buffer.len() {
            // Wait for the peripheral to tell us that we can
            // read from the RX register
            loop {
                let status = volatile!(self.registers.status);
                // TODO: define these constants somewhere
                if status & 0x00000001 == 0x00000001 {
                    break;
                }
            }
            buffer[i] = (volatile!(self.registers.receive_holding)) as u8;
        }
    }
}
