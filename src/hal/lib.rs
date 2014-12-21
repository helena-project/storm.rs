#![crate_name = "hal"]
#![crate_type = "rlib"]
#![no_std]
#![feature(macro_rules, globs)]

extern crate core;

mod std {
    pub use core::*;
}

pub mod ast;
pub mod gpio;
pub mod nvic;
pub mod pm;
pub mod spi;
pub mod usart;
