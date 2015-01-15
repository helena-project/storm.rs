use core::prelude::*;
use core::intrinsics::{volatile_load,volatile_store};

#[allow(dead_code)]
struct PmRegisters {
    mcctrl : u32,
    cpusel : u32,
    reserved0 : u32,
    pbasel : u32,
    pbbsel : u32,
    pbcsel : u32,
    pbdsel : u32,
    reserved1 : u32,
    //0x020
    cpumask : u32,
    hsbmask : u32,
    pbamask : u32,
    pbbmask : u32,
    pbcmask : u32,
    pbdmask : u32,
    reserved2 : [u32;2],
    //0x040
    pbadivmask : u32,
    reserved3 : [u32;4],
    cfdctrl : u32,
    unlock : u32,
    reserved4 : u32,
    //0x60
    reserved5 : [u32;24],
    //0xC0
    ier : u32,
    idr : u32,
    imr : u32,
    isr : u32,
    icr : u32,
    sr : u32,
    reserved6 : [u32;2],
    //0x100
    reserved7 : [u32;24],
    //0x160
    ppcr : u32,
    reserved8 : [u32;7],
    //0x180
    rcause : u32,
    wcause : u32,
    awen : u32,
    protctrl : u32,
    reserved9 : u32,
    fastsleep : u32,
    reserved10 : [u32;2],
    //0x200
    config : u32,
    version : u32
}

pub const PM_BASE : isize = 0x400E0000;

static mut PM : *mut PmRegisters = PM_BASE as *mut PmRegisters;

pub enum Clock {
    RCSYS = 0,
    OSC0,
    PLL,
    DFLL,
    RC80M,
    RCFAST,
    RC1M
}

impl Copy for Clock {}

unsafe fn unlock(register_offset : u32) {
    volatile_store(&mut (*PM).unlock, 0xAA000000 | register_offset);
}

pub fn select_main_clock(clock : Clock) {
    unsafe {
        volatile_store(&mut(*PM).mcctrl, clock as u32 & 0xFF);
    }
}

pub fn enable_pba_clock(clock : usize) {
    unsafe {
        unlock(0x028);

        let val = volatile_load(&(*PM).pbamask) | (1 << clock);
        volatile_store(&mut (*PM).pbamask, val);
    }
}

