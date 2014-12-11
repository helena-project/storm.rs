use core::intrinsics;
use nvic;

#[repr(C, packed)]
pub struct Ast {
    cr : u32,
    cv : u32,
    sr : u32,
    pub scr : u32,
    ier : u32,
    idr : u32,
    imr : u32,
    wer : u32,
    //0x20
    ar0 : u32,
    ar1 : u32,
    reserved0 : [u32, ..2],
    pir0 : u32,
    pir1 : u32,
    reserved1 : [u32, ..2],
    //0x40
    clock : u32,
    dtr : u32,
    eve : u32,
    evd : u32,
    evm : u32,
    calv : u32
    //we leave out parameter and version
}

pub const AST_BASE : int = 0x400F0800;

static mut GAst : *mut Ast = AST_BASE as *mut Ast;

#[repr(uint)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4
}

pub fn initialize() {
    select_clock(Clock::ClockRCSys);
    set_prescalar(0);
    clear_alarm();
}

pub fn clock_busy() -> bool {
    unsafe {
        intrinsics::volatile_load(&(*GAst).sr) & (1 << 28) != 0
    }
}


pub fn busy() -> bool {
    unsafe {
        intrinsics::volatile_load(&(*GAst).sr) & (1 << 24) != 0
    }
}

// Clears the alarm bit in the status register (indicating the alarm value
// has been reached).
pub fn clear_alarm() {
    while busy() {}
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).scr, 1 << 8);
    }
}

// Clears the per0 bit in the status register (indicating the alarm value
// has been reached).
pub fn clear_periodic() {
    while busy() {}
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).scr, 1 << 16);
    }
}


pub fn select_clock(clock : Clock) {
    unsafe {
      // Disable clock by setting first bit to zero
      let enb = intrinsics::volatile_load(&(*GAst).clock) ^ 1;
      intrinsics::volatile_store(&mut (*GAst).clock, enb);
      while clock_busy() {}

      // Select clock
      intrinsics::volatile_store(&mut (*GAst).clock, (clock as u32) << 8);
      while clock_busy() {}

      // Re-enable clock
      let enb = intrinsics::volatile_load(&(*GAst).clock) | 1;
      intrinsics::volatile_store(&mut (*GAst).clock, enb);
    }
}

pub fn enable() {
    while busy() {}
    unsafe {
        let cr = intrinsics::volatile_load(&(*GAst).cr) | 1;
        intrinsics::volatile_store(&mut (*GAst).cr, cr);
    }
}

pub fn disable() {
    while busy() {}
    unsafe {
        let cr = intrinsics::volatile_load(&(*GAst).cr) & !1;
        intrinsics::volatile_store(&mut (*GAst).cr, cr);
    }
}

pub fn set_prescalar(val : u8) {
    while busy() {}
    unsafe {
        let cr = intrinsics::volatile_load(&(*GAst).cr) | (val as u32) << 16;
        intrinsics::volatile_store(&mut (*GAst).cr, cr);
    }
}

pub fn enable_alarm_irq() {
    nvic::enable(nvic::NvicIdx::ASTALARM);
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).ier, 1 << 8);
    }
}

pub fn disable_alarm_irq() {
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).idr, 1 << 8);
    }
}

pub fn enable_ovf_irq() {
    nvic::enable(nvic::NvicIdx::ASTOVF);
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).ier, 1);
    }
}

pub fn disable_ovf_irq() {
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).idr, 1);
    }
}

pub fn enable_periodic_irq() {
    nvic::enable(nvic::NvicIdx::ASTPER);
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).ier, 1 << 16);
    }
}

pub fn disable_periodic_irq() {
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).idr, 1 << 16);
    }
}

pub fn set_periodic_interval(interval : u32) {
    while busy() {}
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).pir0, interval);
    }
}

pub fn get_counter() -> u32 {
    while busy() {}
    unsafe {
        intrinsics::volatile_load(&(*GAst).cv)
    }
}


pub fn set_counter(value : u32) {
    while busy() {}
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).cv, value);
    }
}

pub fn set_alarm(tics : u32) {
    while busy() {}
    unsafe {
        intrinsics::volatile_store(&mut (*GAst).ar0, tics);
    }
}

pub fn start_periodic() {
    disable();
    enable_periodic_irq();
    set_periodic_interval(14);
    clear_periodic();
    set_counter(0);
    enable();
}

pub fn stop_periodic() {
    disable_periodic_irq();
}

