use core::intrinsics;

#[repr(C, packed)]
struct Nvic {
    iser : [u32, ..28]
}

pub enum NvicIdx {
    HFLASHC = 0,
    PDCA0,
    PDCA1,
    PDCA2,
    PDCA3,
    PDCA4,
    PDCA5,
    PDCA6,
    PDCA7,
    PDCA8,
    PDCA9,
    PDCA10,
    PDCA11,
    PDCA12,
    PDCA13,
    PDCA14,
    PDCA15,
    CRCCU,
    USBC,
    PEVCTR,
    PEVCOV,
    AESA,
    PM,
    SCIF,
    FREQM,
    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,
    GPIO5,
    GPIO6,
    GPIO7,
    GPIO8,
    GPIO9,
    GPIO10,
    GPIO11,
    BPM,
    BSCIF,
    ASTALARM,
    ASTPER,
    ASTOVF,
    ASTREADY,
    ASTCLKREADY,
    WDT,
    EIC1,
    EIC2,
    EIC3,
    EIC4,
    EIC5,
    EIC6,
    EIC7,
    EIC8,
    IISC,
    SPI,
    TC00,
    TC01,
    TC02,
    TC10,
    TC11,
    TC12,
    TWIM0,
    TWIS0,
    TWIM1,
    TWIS1,
    USART0,
    USART1,
    USART2,
    USART3,
    ADCIFE,
    DACC,
    ACIFC,
    TRNG,
    PARC,
    CATB,
    TWIM2,
    TWIM3,
    LCDCA
}

pub fn enable(signal : NvicIdx) {
    let nvic_addr : u32 = 0xe000e100;
    let nvic = unsafe { &mut *(nvic_addr as *mut Nvic)};
    let int = signal as uint;

    unsafe {
        intrinsics::volatile_store(&mut nvic.iser[int / 32], 1 << (int & 31));
    }
}

