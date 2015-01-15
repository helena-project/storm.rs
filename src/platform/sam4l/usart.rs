use core::intrinsics;
use hil::uart;

#[repr(C, packed)]
#[allow(dead_code)]
struct UsartRegisters {
    cr : u32,
    mr : u32,
    ier : u32,
    idr : u32,
    imr : u32,
    csr : u32,
    rhr : u32,
    thr : u32,
    //0x20
    brgr : u32,
    rtor : u32,
    ttgr : u32,
    reserved0 : [u32;5],
    //0x40
    fidi : u32,
    ner : u32,
    reserved1 : u32,
    ifr : u32,
    man : u32,
    linmr : u32,
    linir : u32,
    linbrr : u32,
    wpmr : u32,
    wpsr : u32,
    version : u32
}

const USART_SIZE : isize = 0x4000;
const USART_BASE_ADDRESS : isize = 0x40024000;

#[derive(Copy)]
pub enum BaseAddr {
    USART0 = USART_BASE_ADDRESS,
    USART1 = USART_BASE_ADDRESS + USART_SIZE * 1,
    USART2 = USART_BASE_ADDRESS + USART_SIZE * 2,
    USART3 = USART_BASE_ADDRESS + USART_SIZE * 3,
}

pub struct USART {
    regs: &'static mut UsartRegisters
}

impl USART {
    pub fn new(base: BaseAddr) -> USART {
        USART {
            regs: unsafe { intrinsics::transmute(base) }
        }

    }

    pub fn enable_rx(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut self.regs.cr, 1 << 4);
        }
    }

    pub fn disable_rx(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut self.regs.cr, 1 << 5);
        }
    }

    pub fn enable_tx(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut self.regs.cr, 1 << 6);
        }
    }

    pub fn disable_tx(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut self.regs.cr, 1 << 7);
        }
    }

    pub fn rx_ready(&self) -> bool {
        unsafe {
            intrinsics::volatile_load(&self.regs.csr) & 0x1 != 0
        }
    }

    pub fn tx_ready(&self) -> bool {
        unsafe {
            intrinsics::volatile_load(&self.regs.csr) & 0x2 != 0
        }
    }

    pub fn set_baud_rate(&mut self, rate : u32) {
        let cd = 48000000 / (16 * rate);
        unsafe {
            intrinsics::volatile_store(&mut self.regs.brgr, cd);
        }
    }

    pub fn set_mode(&mut self, mode : u32) {
        unsafe {
            intrinsics::volatile_store(&mut self.regs.mr, mode);
        }
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
        self.set_mode(mode);
        self.set_baud_rate(params.baud_rate);
        // This is just copied from TinyOS, not exactly sure how to
        // generalize
        unsafe {
            intrinsics::volatile_store(&mut self.regs.ttgr, 4);
        }
    }

    fn toggle_tx(&mut self, enable : bool) {
        if enable {
            self.enable_tx();
        } else {
            self.disable_tx();
        }
    }

    fn toggle_rx(&mut self, enable : bool) {
        if enable {
            self.enable_rx();
        } else {
            self.disable_rx();
        }
    }

    fn send_byte(&mut self, b : u8) {
        unsafe {
            while !self.tx_ready() {}
            intrinsics::volatile_store(&mut self.regs.thr, b as u32);
        }
    }
}

