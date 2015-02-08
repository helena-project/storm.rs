#![crate_name = "apps"]
#![crate_type = "rlib"]
#![allow(unused,unconditional_recursion)] // See Rust issue #21705
#![feature(core)]
#![no_std]

extern crate core;

macro_rules! register_app {
    ($section:expr, $init_func:expr) => (
        #[link_section = $section]
        pub static RUST_BLINK_INIT: fn() = $init_func;
    );
}

pub mod boop;
mod commands;
mod std {
    pub use core::*;
}
