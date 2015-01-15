#![crate_name = "platform"]
#![crate_type = "rlib"]
#![allow(unstable)]
#![no_std]

extern crate core;
extern crate hil;

mod std {
    pub use core::*;
}

pub mod sam4l;
