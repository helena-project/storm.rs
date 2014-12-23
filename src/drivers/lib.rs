#![crate_name = "drivers"]
#![crate_type = "rlib"]
#![no_std]
#![feature(globs)]

extern crate core;
// There should actually be a level of indirection here, since we don't want to
// expose unsafe hardware drivers to platform drivers. But for now...
extern crate hal;

pub mod flash_attr;
