#![crate_name = "hil"]
#![crate_type = "rlib"]
#![allow(unstable)]
#![no_std]

extern crate core;

pub use uart::*;
pub use gpio::*;

mod std {
    pub use core::*;
}

pub mod gpio;
pub mod i2c;
pub mod spi;
pub mod timer;
pub mod uart;
