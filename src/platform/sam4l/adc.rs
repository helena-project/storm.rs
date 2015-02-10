use core::intrinsics;
use hil::adc;
use sam4l::pm::{self, Clock, PBAClock};

#[repr(C, packed)]
#[allow(dead_code,missing_copy_implementations)]
pub struct AdcRegisters { // From page 1092
    cr: usize,         // Control register      (0x00)
    mr: usize,         // Mode register         (0x04)
    seqr1: usize,      // Chan. sequence reg. 1 (0x08)
    seqr2: usize,      // Chan. sequence reg. 2 (0x0c)
    cher: usize,       // Channel enable reg.   (0x10)
    chdr: usize,       // Channel disable reg.  (0x14)
    chsr: usize,       // Channel enable reg.   (0x18)
    reserved0: usize,  // reserved              (0x1c)
    lcdr: usize,       // Last converted data   (0x20)
    ier:  usize,       // Interrupt enable reg. (0x24)
    idr:  usize,       // Interrupt disable     (0x28)
    imr:  usize,       // Interrupt mask reg.   (0x2c)
    isr:  usize,       // Interrupt status reg. (0x30)
    reserved1: [usize; 2], // reserved          (0x34-0x38)
    over: usize,       // Overrun status reg.   (0x3c)
    emr:  usize,       // Extended mode reg.    (0x40)
    cwr:  usize,       // Compare window reg.   (0x44)
    cgr:  usize,       // Channel gain reg.     (0x48)
    cor:  usize,       // Channel offset reg.   (0x4c)
    cdr: [usize; 16],  // Chan. data registers  (0x50-0x8c)
    reserved2: usize,  // reserved              (0x90)
    acr: usize,        // Analog control reg.   (0x94)
    reserved3: [usize; 72], // reserved         (0x98-0xe0)
    wpmr: usize,       // Write protect mode    (0xe4)
    wpsr: usize        // Write protect status  (0xe8)
}

pub const BASE_ADDRESS: usize = 0x40038000;

// -1 means no channel active
static mut WHICH_ACTIVE : isize = -1;
static mut BUSY: bool = false;

#[allow(missing_copy_implementations)]
pub struct ADC {
    chan: isize,
    enabled: bool
}

#[derive(Copy)]
pub enum CHAN {
    Enable = 0,
    Disable = 1
}

fn enable() {
    // Enable ADC MCK in PMC
    pm::enable_clock(Clock::PBA(PBAClock::ADCIFE));
//    let regs: &mut AdcRegisters = unsafe {
//        intrinsics::transmute(BASE_ADDRESS)
//    };
    // Do we need to write 2 to TRANSFER field of
    // mr register? (page 1096)?
    //    volatile!(regs.mr = 2 << 28);
}

fn disable() {
    // Disable SPI Clock
    pm::disable_clock(Clock::PBA(PBAClock::ADCIFE));
}

fn sample(chan: isize) -> u16 {
    let regs: &mut AdcRegisters = unsafe {
        intrinsics::transmute(BASE_ADDRESS)
    };
    volatile!(regs.cher = 1 << chan); // Enable channel
    volatile!(regs.cr = 2);           // Initiate conversion
    while (volatile!(regs.isr) & (1 << chan)) == 0 {} // wait
    let val = volatile!(regs.lcdr) & 0xfff;  // Get result
    volatile!(regs.chdr = 0xffff);     // Disable all channels
    val as u16
}

impl adc::ADCMaster for ADC {
    fn enable(&mut self) -> bool {
        if self.enabled {  // Already enabled
            return true;
        } else {
            let busy = unsafe {
                let state  = &mut BUSY as *mut bool;
                intrinsics::atomic_xchg(state, true)
            };
            if busy == false {
                self.enabled = true;
                unsafe {
                    let which_active = &mut WHICH_ACTIVE as *mut isize;
                    intrinsics::atomic_store(which_active, self.chan)
                };
                enable();
                true
            } else {
                false
            }
        }
    }

    fn disable(&mut self) {
        if !self.enabled {
            return
        }
        disable();
        unsafe {
            let which_active = &mut WHICH_ACTIVE as *mut isize;
            intrinsics::atomic_store(which_active, -1);
            let state  = &mut BUSY as *mut bool;
            intrinsics::atomic_store(state, false)
        };
        self.enabled = false;
    }

    fn is_enabled(&self) -> bool {
       self.enabled
    }

    fn sample(&mut self) -> u16 {
        if !self.is_enabled() {
            0
        } else {
            sample(self.chan)
        }
    }
}

