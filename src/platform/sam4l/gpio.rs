use core::prelude::*;
use core::intrinsics;
use hil;

#[repr(C, packed)]
struct Register {
    val: u32,
    set: u32,
    clear: u32,
    toggle: u32
}

#[repr(C, packed)]
struct RegisterRO {
    val: u32,
    reserved: [u32; 3]
}

#[repr(C, packed)]
struct RegisterRC {
    val: u32,
    reserved0: u32,
    clear: u32,
    reserved1: u32
}

#[repr(C, packed)]
struct GPIOPortRegisters {
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
    reserved4: [u32; 26],
    parameter: u32,
    version: u32,
}

#[derive(Copy)]
pub enum PeripheralFunction {
    A, B, C, D, E, F, G, H
}


const BASE_ADDRESS: usize = 0x400E1000;
const SIZE: usize = 0x200;

repeated_enum!(
pub enum GPIOPort {
    GPIO * 3
});

repeated_enum!(
pub enum Location {
    GPIOPin * 32
});

#[derive(Copy)]
pub struct Params {
    pub location: Location,
    pub port: GPIOPort,
    pub function: Option<PeripheralFunction>
}

pub struct GPIOPin {
    port: &'static mut GPIOPortRegisters,
    number: u8,
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
// checks as soon as possible. Here, for example, we chould check that 'pin' is
// valid and panic before continuing to boot.
impl GPIOPin {
    pub fn new(params: Params) -> GPIOPin {
        let address = BASE_ADDRESS + (params.port as usize) * SIZE;
        let pin_number = params.location as u8;

        let mut pin = GPIOPin {
            port: unsafe { intrinsics::transmute(address) },
            number: pin_number,
            pin_mask: 1 << (pin_number as u32)
        };

        if params.function.is_some() {
            pin.select_peripheral(params.function.unwrap());
        }

        pin
    }

    pub fn select_peripheral(&mut self, function: PeripheralFunction) {
        let (f, n) = (function as u32, self.number as u32);
        let (bit0, bit1, bit2) = (f & 0b1, (f & 0b10) >> 1, (f & 0b100) >> 2);

        // clear GPIO enable for pin
        volatile!(self.port.gper.clear = self.pin_mask);

        // Set PMR0-2 according to passed in peripheral
        volatile!(self.port.pmr0.val = bit0 << n);
        volatile!(self.port.pmr1.val = bit1 << n);
        volatile!(self.port.pmr2.val = bit2 << n);
    }
}

impl hil::GPIOPin for GPIOPin {
    fn enable_output(&mut self) {
        volatile!(self.port.gper.set = self.pin_mask);
        volatile!(self.port.oder.set = self.pin_mask);
        volatile!(self.port.ster.clear = self.pin_mask);
    }

    fn read(&self) -> bool {
        (volatile!(self.port.pvr.val) & self.pin_mask) > 0
    }

    port_register_fn!(toggle, ovr, toggle);
    port_register_fn!(set, ovr, set);
    port_register_fn!(clear, ovr, clear);
}
