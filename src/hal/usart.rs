use core::intrinsics;
use core::str::StrPrelude;

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
    reserved0 : [u32,..5],
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

const USART_SIZE : int = 0x4000;
const USART_BASE_ADDRESS : int = 0x40024000;

pub enum USART {
    UART0 = USART_BASE_ADDRESS,
    UART1 = USART_BASE_ADDRESS + USART_SIZE * 1,
    UART2 = USART_BASE_ADDRESS + USART_SIZE * 2,
    UART3 = USART_BASE_ADDRESS + USART_SIZE * 3,
}

macro_rules! usart (
    ($addr : expr) => (
        unsafe {
            &mut *($addr as u32 as *mut UsartRegisters)
        }
    );
)

impl USART {
    pub fn init_uart(self) {
        let dev = usart!(self);
        let mode = 0 /* mode */ |
                   0 << 4 /*USCLKS*/ |
                   3 << 6 /* CHRL 8 bits */ |
                   4 << 9 /* no parity */ |
                   0 << 12 /* NBSTOP 1 bit */;
        unsafe {
            intrinsics::volatile_store(&mut dev.mr, mode);
            intrinsics::volatile_store(&mut dev.ttgr, 4);
        }
    }

    pub fn enable_rx(self) {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_store(&mut dev.cr, 1 << 4);
        }
    }

    pub fn disable_rx(self) {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_store(&mut dev.cr, 1 << 5);
        }
    }

    pub fn enable_tx(self) {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_store(&mut dev.cr, 1 << 6);
        }
    }

    pub fn disable_tx(self) {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_store(&mut dev.cr, 1 << 7);
        }
    }

    pub fn rx_ready(self) -> bool {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_load(&dev.csr) & 0x1 != 0
        }
    }

    pub fn tx_ready(self) -> bool {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_load(&dev.csr) & 0x2 != 0
        }
    }

    pub fn send_byte(self, b : u8) {
        let dev = usart!(self);
        unsafe {
            intrinsics::volatile_store(&mut dev.thr, b as u32);
        }
    }

    pub fn print(self, s : &str) {
        for b in s.bytes() {
            while !self.tx_ready() {}
            self.send_byte(b);
        }
    }

    pub fn set_baud_rate(self, rate : u32) {
      let dev = usart!(self);
      let cd = 48000000 / (16 * rate);
      unsafe {
          intrinsics::volatile_store(&mut dev.brgr, cd);
      }
    }
}
