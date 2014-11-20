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

macro_rules! ast(
  () => {
    unsafe { &mut *(AST_BASE as u32 as *mut Ast) };
  }
)

pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4
}

impl Ast {

    pub fn initialize(&mut self) {
        self.select_clock(ClockRCSys);
        self.set_prescalar(1);
        self.clear_alarm();
    }

    pub fn clock_busy(&self) -> bool {
        unsafe {
            intrinsics::volatile_load(&self.sr) & (1 << 28) != 0
        }
    }


    pub fn busy(&self) -> bool {
        unsafe {
            intrinsics::volatile_load(&self.sr) & (1 << 24) != 0
        }
    }

    // Clears the alarm bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_alarm(&mut self) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut self.scr, 1 << 8);
        }
    }

    // Clears the per0 bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_periodic(&mut self) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut self.scr, 1 << 16);
        }
    }


    pub fn select_clock(&mut self, clock : Clock) {
        unsafe {
          // Disable clock by setting first bit to zero
          let enb = intrinsics::volatile_load(&self.clock) ^ 1;
          intrinsics::volatile_store(&mut (self.clock), enb);
          while self.clock_busy() {}

          // Select clock
          intrinsics::volatile_store(&mut (self.clock), (clock as u32) << 8);
          while self.clock_busy() {}

          // Re-enable clock
          let enb = intrinsics::volatile_load(&self.clock) | 1;
          intrinsics::volatile_store(&mut (self.clock), enb);
        }
    }

    pub fn enable(&mut self) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&self.cr) | 1;
            intrinsics::volatile_store(&mut self.cr, cr);
        }
    }

    pub fn disable(&mut self) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&self.cr) ^ 1;
            intrinsics::volatile_store(&mut self.cr, cr);
        }
    }

    pub fn set_prescalar(&mut self, val : u8) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&self.cr) | (val as u32) << 16;
            intrinsics::volatile_store(&mut self.cr, cr);
        }
    }

    pub fn enable_periodic_irq(&mut self) {
        nvic::enable(nvic::ASTPER);
        unsafe {
            intrinsics::volatile_store(&mut self.ier, 1 << 16);
        }
    }

    pub fn disable_periodic_irq(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut self.idr, 1 << 16);
        }
    }

    pub fn set_periodic_interval(&mut self, interval : u32) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut self.pir0, interval);
        }
    }

    pub fn set_counter(&mut self, value : u32) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut self.cv, value);
        }
    }

    pub fn start_periodic(&mut self) {
        self.disable();
        self.enable_periodic_irq(); 
        self.set_periodic_interval(15);
        self.clear_periodic();
        self.set_counter(0);
        self.enable();
    }

    pub fn stop_periodic(&mut self) {
        self.disable_periodic_irq(); 
    }
}

pub fn initialize() {
    let ast = unsafe { &mut *(AST_BASE as u32 as *mut Ast) };
    ast.initialize();
}

pub fn start_periodic() {
    let ast = ast!();
    ast.start_periodic();
}
