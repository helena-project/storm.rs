use core::intrinsics;
use hil::gpio;

#[repr(C, packed)]
struct Register {
    val: u32,
    set: u32,
    clear: u32,
    toggle: u32
}

#[repr(C, packed)]
struct RegisterRO {
    read: u32,
    reserved: [u32; 3]
}

#[repr(C, packed)]
struct RegisterRC {
    read: u32,
    reserved0: u32,
    clear: u32,
    reserved1: u32
}

#[repr(C, packed)]
struct GPIOPort {
    gper: Register,
    pmr0: Register,
    pmr1: Register,
    pmr2: Register,
    oder: Register,
    ovr: Register,
    pvr: RegisterRO,
    puer: Register,
    pder: Register,
    ier: Register,
    imr0: Register,
    imr1: Register,
    gfer: Register,
    ifr: RegisterRC,
    reserved0: [u32; 8],
    ocdr0: Register,
    ocdr1: Register,
    reserved1: [u32; 4],
    osrr0: Register,
    reserved2: [u32; 8],
    ster: Register,
    reserved3: [u32; 4],
    ever: Register,
    // reserved4: [u32; 26],
    // PARAMETER: u32,
    // VERSION: u32,
}

const BASE_ADDRESS: usize = 0x400E1000;
const SIZE: usize = 0x200;

#[derive(Copy)]
pub enum Location {
    GPIO0 = 0,
    GPIO1 = 1,
    GPIO2 = 2,
}

#[derive(Copy)]
pub struct Params {
    pub location: Location,
    pub pin: u8,
}

struct GPIO {
    port: &'static mut GPIOPort,
    pin: u8,
    pin_mask: u32
}

macro_rules! port_register_fn {
    ($name:ident, $reg:ident, $option:ident) => (
        fn $name(&mut self) {
            volatile!(self.port.$reg.$option = self.pin_mask);
        }
    );
}

// Note: Perhaps the 'new' function should return Result<T> to do simple init
// checks quickly. Here, for example, we chould check that 'pin' is valid and
// panic before continuing to boot.
impl GPIO {
    pub fn new(params: Params) -> GPIO {
        let address = BASE_ADDRESS + (params.location as usize) * SIZE;

        GPIO {
            port: unsafe { intrinsics::transmute(address) },
            pin: params.pin,
            pin_mask: 1 << params.pin
        }
    }
}

impl gpio::Pin for GPIO {
    fn make_output(&mut self) {
        volatile!(self.port.gper.set = self.pin_mask);
        volatile!(self.port.oder.set = self.pin_mask);
        volatile!(self.port.ster.clear = self.pin_mask);
    }

    fn select_peripheral(&mut self, function: gpio::PeripheralFunction) {
        let (f, p) = (function as u32, self.pin as u32);
        let (bit0, bit1, bit2) = (f & 0b1, (f & 0b10) >> 1, (f & 0b100) >> 2);

        // clear GPIO enable for pin
        volatile!(self.port.gper.clear = self.pin_mask);

        // Set PMR0-2 according to passed in peripheral
        volatile!(self.port.pmr0.val = bit0 << p);
        volatile!(self.port.pmr1.val = bit1 << p);
        volatile!(self.port.pmr2.val = bit2 << p);
    }

    port_register_fn!(toggle, ovr, toggle);
    port_register_fn!(set, ovr, set);
    port_register_fn!(clear, ovr, clear);
}
