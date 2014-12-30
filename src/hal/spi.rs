use core::intrinsics::*;
use hil::spi;

#[repr(C, packed)]
#[allow(dead_code)]
struct SpiRegisters {
    cr : uint,
    mr : uint,
    rdr : uint,
    tdr : uint,
    sr : uint,
    ier : uint,
    idr : uint,
    imr : uint,
    //0x20
    reserved0 : [uint,..4],
    csr : [uint,..4],
    //0x40
    reserved1 : [uint,..41],
    wpcr : uint,
    wpsr : uint
    //we leave out parameter and version
}

pub const SPI_BASE : uint = 0x40008000;

static mut GSPI : *mut SpiRegisters = SPI_BASE as *mut SpiRegisters;

#[allow(missing_copy_implementations)]
pub struct SPI {
    pub cs: uint
}

#[deriving(Copy)]
pub enum MSTR {
    Master = 1,
    Slave = 0
}

#[deriving(Copy)]
pub enum PS {
    Fixed = 0,
    Variable = 1
}

#[deriving(Copy)]
pub enum RXFIFO {
    Disable = 0,
    Enable = 1
}

#[deriving(Copy)]
pub enum MODFAULT {
    Enable = 0,
    Disable = 1
}

pub fn enable() {
    unsafe {
        volatile_store(&mut(*GSPI).cr, 1);
    }
}

pub fn disable() {
    unsafe {
        volatile_store(&mut(*GSPI).cr, 2);
    }
}

pub fn set_mode(mstr : MSTR, ps : PS, rxfifo : RXFIFO, modf : MODFAULT) {
    let mode = (mstr as uint) | ps as uint << 1 | rxfifo as uint << 6 |
                modf as uint << 4;
    unsafe {
        volatile_store(&mut(*GSPI).mr, mode);
    }
}

impl spi::SPI for SPI {
    fn set_baud_rate(&self, divisor : u8) {
        unsafe {
            let mut csr = volatile_load(&(*GSPI).csr[self.cs]);
            csr = (divisor as uint << 8) | (csr & 0xffff00ff);
            volatile_store(&mut(*GSPI).csr[self.cs], csr);
        }
    }

    fn set_mode(&self, mode : spi::Mode) {
        unsafe {
            let mut csr = volatile_load(&(*GSPI).csr[self.cs]);
            csr = (mode as uint) | (csr & 0xfffffffc);
            volatile_store(&mut(*GSPI).csr[self.cs], csr);
        }
    }

    fn write_read(&self, data : u16, lastxfer : bool) -> u16 {
        unsafe {
            let tdr = (!(1 << self.cs) & 0xf) << 16 |
                      data as uint |
                      lastxfer as uint << 24;
            volatile_store(&mut(*GSPI).tdr, tdr);

            while (volatile_load(&(&*GSPI).sr) & 1) != 1 {}
            volatile_load(&(*GSPI).rdr) as u16
        }
    }
}

