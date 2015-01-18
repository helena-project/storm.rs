use core::intrinsics;
use hil::uart;

// This should likely be moved to a better location. Maybe hil?
macro_rules! volatile {
    ($item:expr) => ({
        use core::intrinsics::volatile_load;
        unsafe { volatile_load(&$item) }
    });

    ($item:ident = $value:expr) => ({
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, $value); }
    });
}

#[repr(C, packed)]
#[allow(dead_code)]
struct UsartRegisters {
    cr: u32,
    mr: u32,
    ier: u32,
    idr: u32,
    imr: u32,
    csr: u32,
    rhr: u32,
    thr: u32,
    //0x20
    brgr: u32,
    rtor: u32,
    ttgr: u32,
    reserved0: [u32; 5],
    //0x40
    fidi: u32,
    ner: u32,
    reserved1: u32,
    ifr: u32,
    man: u32,
    linmr: u32,
    linir: u32,
    linbrr: u32,
    wpmr: u32,
    wpsr: u32,
    version: u32
}

const SIZE: usize = 0x4000;
const BASE_ADDRESS: usize = 0x40024000;

#[derive(Copy)]
pub enum Location {
    USART0 = 0,
    USART1 = 1,
    USART2 = 2,
    USART3 = 3,
}

#[derive(Copy)]
pub struct Params {
    pub location: Location,
}

pub struct USART {
    regs: &'static mut UsartRegisters
}

impl USART {
    pub fn new(params: Params) -> USART {
        let address = BASE_ADDRESS + (params.location as usize) * SIZE;

        USART {
            regs: unsafe { intrinsics::transmute(address) }
        }
    }

    fn set_baud_rate(&mut self, baud_rate: u32) {
        let cd = 48000000 / (16 * baud_rate);
        volatile!(self.regs.brgr = cd);
    }

    // This can be made safe by having a struct represent the mode register,
    // with enums when there are choices and not just numbers, and passing the
    // struct to this function. As is, it's too easy to make a mistake.
    unsafe fn set_mode(&mut self, mode: u32) {
        volatile!(self.regs.mr = mode);
    }

    pub fn rx_ready(&self) -> bool {
        volatile!(self.regs.csr) & 0b1 != 0
    }

    pub fn tx_ready(&self) -> bool {
        volatile!(self.regs.csr) & 0b10 != 0
    }
}

impl uart::UART for USART {
    fn init(&mut self, params: uart::UARTParams) {
        let parity = match params.parity {
            uart::Parity::EVEN => 0,
            uart::Parity::ODD => 1,
            uart::Parity::FORCE0 => 2,
            uart::Parity::FORCE1 => 3,
            uart::Parity::NONE => 4,
            uart::Parity::MULTIDROP => 5
        };

        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode : u32 = 0 /* mode */ |
                   0 << 4 /*USCLKS*/ |
                   chrl << 6 /* CHRL 8 bits */ |
                   parity << 9 /* no parity */ |
                   0 << 12 /* NBSTOP 1 bit */;

        unsafe { self.set_mode(mode); }
        self.set_baud_rate(params.baud_rate);
        // Copied from TinyOS, not exactly sure how to generalize
        volatile!(self.regs.ttgr = 4);
    }

    fn send_byte(&mut self, byte: u8) {
        while !self.tx_ready() {}
        volatile!(self.regs.thr = byte as u32);
    }

    fn toggle_rx(&mut self, enable: bool) {
        if enable {
            volatile!(self.regs.cr = 1 << 4);
        } else {
            volatile!(self.regs.cr = 1 << 5);
        }
    }

    fn toggle_tx(&mut self, enable: bool) {
        if enable {
            volatile!(self.regs.cr = 1 << 6);
        } else {
            volatile!(self.regs.cr = 1 << 7);
        }
    }
}

