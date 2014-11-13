use nvic;
use intrinsics;

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

enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4
}

impl Ast {

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
          self.clock |= 1;
        }
    }

    pub fn setup(&mut self) {
        // Select clock
        self.select_clock(ClockRCSys);

        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut self.cr, 1 | 1 << 16)
        }

        nvic::enable(40);
    }

    pub fn start_periodic(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut self.ier, 1 << 16);

            while self.busy() {}
            intrinsics::volatile_store(&mut self.pir0, 15);

            while self.busy() {}
            intrinsics::volatile_store(&mut self.scr, 1 << 16);

            while self.busy() {}
            intrinsics::volatile_store(&mut self.cv, 0);
        }
    }

    pub fn stop_periodic(&mut self) {
        unsafe { intrinsics::volatile_store(&mut self.idr, 1 << 16); }
    }
}

