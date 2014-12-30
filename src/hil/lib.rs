#![crate_name = "hil"]
#![crate_type = "rlib"]
#![no_std]
#![feature(globs)]

extern crate core;

mod std {
    pub use core::*;
}

pub mod gpio;
pub mod spi;
pub mod timer;
