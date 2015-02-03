use core::intrinsics;
use hil::spi;
use sam4l::pm;

#[repr(C, packed)]
#[allow(dead_code,missing_copy_implementations)]
pub struct SpiRegisters {
    cr: usize,
    mr: usize,
    rdr: usize,
    tdr: usize,
    sr: usize,
    ier: usize,
    idr: usize,
    imr: usize,
    //0x20
    reserved0: [usize; 4],
    csr: [usize; 4],
    //0x40
    reserved1: [usize; 41],
    wpcr: usize,
    wpsr: usize
    //we leave out parameter and version
}

pub const BASE_ADDRESS: usize = 0x40008000;

static mut NUM_ENABLED : isize = 0;

#[allow(missing_copy_implementations)]
pub struct SPI {
    registers: &'static mut SpiRegisters,
    pcs: usize,
    enabled: bool
}

#[derive(Copy)]
pub enum MSTR {
    Master = 1,
    Slave = 0
}

#[derive(Copy)]
pub enum PS {
    Fixed = 0,
    Variable = 1
}

#[derive(Copy)]
pub enum RXFIFO {
    Disable = 0,
    Enable = 1
}

#[derive(Copy)]
pub enum MODFAULT {
    Enable = 0,
    Disable = 1
}

fn enable() {
    // Enable SPI Clock
    pm::enable_pba_clock(1);

    let nvic : &mut SpiRegisters = unsafe {
        intrinsics::transmute(BASE_ADDRESS)
    };
    volatile!(nvic.cr = 1);
}

fn disable() {
    let nvic : &mut SpiRegisters = unsafe {
        intrinsics::transmute(BASE_ADDRESS)
    };
    volatile!(nvic.cr = 2);

    // Disable SPI Clock
    pm::disable_pba_clock(1);
}

impl spi::SPIMaster for SPI {
    fn enable(&mut self) {
        if self.enabled {
            return
        }

        let res = unsafe {
            let num_enabled = &mut NUM_ENABLED as *mut isize;
            intrinsics::atomic_xadd(num_enabled, 1)
        };
        if res == 1 {
            enable();
        }
    }

    fn disable(&mut self) {
        if !self.enabled {
            return
        }

        let res = unsafe {
            let num_enabled = &mut NUM_ENABLED as *mut isize;
            intrinsics::atomic_xsub(num_enabled, 1)
        };
        if res == 0 {
            disable();
        }
    }

    fn set_baud_rate(&mut self, divisor: u8) {
        let mut csr = volatile!(self.registers.csr[self.pcs]);
        csr = (divisor as usize) << 8 | (csr & 0xffff00ff);
        volatile!(self.registers.csr[self.pcs] = csr);
    }

    fn set_mode(&mut self, mode: spi::Mode) {
        let mut csr = volatile!(self.registers.csr[self.pcs]);
        csr = (mode as usize) | (csr & 0xfffffffc);
        volatile!(self.registers.csr[self.pcs] = csr);
    }

    fn write_read(&mut self, data: u16, lastxfer: bool) -> u16 {
        // lastxfer in bit 24
        // Peripheral Chip Select in bits 16-19
        // data in bottom 16 bits
        let tdr = (!(1 << self.pcs) & 0xf) << 16 |
                  data as usize |
                  (lastxfer as usize) << 24;
        volatile!(self.registers.tdr = tdr);

        while (volatile!(self.registers.sr) & 1) != 1 {}
        volatile!(self.registers.rdr) as u16
    }
}

