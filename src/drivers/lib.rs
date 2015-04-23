#![crate_name = "drivers"]
#![crate_type = "rlib"]
#![feature(core,no_std)]
#![no_std]

extern crate core;
extern crate collections;
extern crate hil;

mod std {
    pub use core::*;
}

// pub mod flash_attr;
pub mod timer;
pub mod uart;
pub mod gpio;
pub mod i2c;
