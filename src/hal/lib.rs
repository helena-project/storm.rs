#![crate_name = "hal"]
#![crate_type = "rlib"]
#![allow(unstable)]
#![no_std]

extern crate core;
extern crate hil;

mod std {
    pub use core::*;
}

pub mod ast;
pub mod gpio;
pub mod nvic;
pub mod pm;
pub mod spi;
pub mod usart;
