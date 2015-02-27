#![crate_name = "hil"]
#![crate_type = "rlib"]
#![feature(core,no_std)]
#![no_std]

extern crate core;

pub use uart::*;
pub use gpio::*;

mod std {
    pub use core::*;
}

pub mod gpio;
pub mod i2c;
pub mod rng;
pub mod spi;
pub mod timer;
pub mod uart;
// pub mod adc;
