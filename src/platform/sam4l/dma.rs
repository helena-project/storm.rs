/*
 * DMA Support for the Atmel SAM4L.
 *
 * Uses the PDCA peripheral.
 */

// use core::prelude::SliceExt;
use core::intrinsics;

use sam4l;

// Listing of all registers for a particular DMA channel.
// Section 16.6 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct DMARegisters {
    memory_address:          usize,
    peripheral_select:       usize,
    transfer_counter:        usize,
    memory_address_reload:   usize,
    transfer_counter_reload: usize,
    control:                 usize,
    mode:                    usize,
    status:                  usize,
    interrupt_enable:        usize,
    interrupt_disable:       usize,
    interrupt_mask:          usize,
    interrupt_status:        usize
}

// The addresses in memory (7.1 of manual) of the PDCA peripheral
const DMA_BASE_ADDR: usize = 0x400A2000;
const SIZE: usize = 0x40;

// Keep track of how many DMA channels are active. When this value is 0 the
// PDCA clock will be turned off.
static mut NUM_ENABLED: isize = 0;

// SAM4L has 16 DMA channels
#[derive(Copy,Clone)]
pub enum DMALocation {
    DMAChannel00,
    DMAChannel01,
    DMAChannel02,
    DMAChannel03,
    DMAChannel04,
    DMAChannel05,
    DMAChannel06,
    DMAChannel07,
    DMAChannel08,
    DMAChannel09,
    DMAChannel10,
    DMAChannel11,
    DMAChannel12,
    DMAChannel13,
    DMAChannel14,
    DMAChannel15
}

// Specify which peripheral to use with the DMA channel.
// RX: Transfer data from peripheral to memory
// TX: Transfer data from memory to peripheral
// Datasheet 16.7 table 16-8
#[allow(non_camel_case_types)]
pub enum DMAPeripheralIdentifiers {
    USART0_RX      = 0,
    USART1_RX      = 1,
    USART2_RX      = 2,
    USART3_RX      = 3,
    SPI_RX         = 4,
    TWIM0_RX       = 5,
    TWIM1_RX       = 6,
    TWIM2_RX       = 7,
    TWIM3_RX       = 8,
    TWIS0_RX       = 9,
    TWIS1_RX       = 10,
    ADCIFE_RX      = 11,
    CATB_RX        = 12,
    IISC_CH0_RX    = 14,
    IISC_CH1_RX    = 15,
    PARC_RX        = 16,
    AESA_RX        = 17,
    USART0_TX      = 18,
    USART1_TX      = 19,
    USART2_TX      = 20,
    USART3_TX      = 21,
    SPI_TX         = 22,
    TWIM0_TX       = 23,
    TWIM1_TX       = 24,
    TWIM2_TX       = 25,
    TWIM3_TX       = 26,
    TWIS0_TX       = 27,
    TWIS1_TX       = 28,
    ADCIFE_TX      = 29,
    CATB_TX        = 30,
    ABDACB_SDR0_TX = 31,
    ABDACB_SDR1_TX = 32,
    IISC_CH0_TX    = 33,
    IISC_CH1_TX    = 34,
    DACC_TX        = 35,
    AESA_TX        = 36,
    LCDCA_ACMDR_TX = 37,
    LCDCA_ABMDR_TX = 38
}

pub enum DMATransferSize {
    BYTE     = 0,
    HALFWORD = 1,
    WORD     = 2
}

// These parameters are passed in from the platform's device tree.
#[derive(Copy,Clone)]
pub struct DMAParams {
    pub location: DMALocation
}

// This is instantiated when an DMA device is created by the device tree.
// This represents an abstraction of the peripheral hardware.
pub struct DMADevice {
    registers: &'static mut DMARegisters,
    clock: sam4l::pm::Clock,
    enabled: bool
}

// Need to implement the `new` function on the DMA device as a constructor.
// This gets called from the device tree.
impl DMADevice {
    pub fn new (params: DMAParams) -> DMADevice {
        let address = DMA_BASE_ADDR + ((params.location as usize) * SIZE);

        // return
        DMADevice {
            registers: unsafe { intrinsics::transmute(address) },
            clock: sam4l::pm::Clock::HSB(sam4l::pm::HSBClock::PDCA),
            enabled: false
        }
    }

    pub fn enable (&mut self) {
        if self.enabled {
            return
        }

        // We are now enabled. This basically marks the clock as in use.
        enable_reference_increment!(NUM_ENABLED, self);

        // Actually set the control register to enable the channel
        volatile!(self.registers.control = 0x1);

        self.enabled = true;
    }

    pub fn disable (&mut self) {
        if !self.enabled {
            return
        }

        // Subtract one user from the reference counter and disable
        // the clock if needed.
        enable_reference_decrement!(NUM_ENABLED, self);

        // Actually set the control register to disable the channel
        volatile!(self.registers.control = 0x2);

        self.enabled = false;
    }

    /// Configure which hardware peripheral this DMA channel should talk
    /// to. This also configures the direction, as TX will write to the
    /// peripheral and RX will read from it.
    pub fn set_peripheral_identifier (&mut self, pid: DMAPeripheralIdentifiers) {
        volatile!(self.registers.peripheral_select = pid as usize);
    }

    /// Synchronous data transfer function. Initiate DMA then wait for it
    /// to finish.
    pub fn transfer_sync (&mut self, destination: &mut u8, size: DMATransferSize, length: u16) {
        self.enable();

        // Write the size of each transferred element
        volatile!(self.registers.mode = size as usize);

        // Configure where the data is going/coming from
        volatile!(self.registers.memory_address = intrinsics::transmute(destination));

        // Configure how many data items we want to move.
        // This will cause the transfer to start.
        volatile!(self.registers.transfer_counter = length as usize);

        // Because this is the sync version, wait for it to finish
        loop {
            let status = volatile!(self.registers.interrupt_status);
            // TRC
            if status & 0x2 == 0x2 {
                break;
            }
        }
    }
}
