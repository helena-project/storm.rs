#![crate_name = "plugins"]
#![crate_type = "dylib"]

#![allow(unstable)]
#![feature(plugin_registrar, quote)]

extern crate syntax;
extern crate rustc;

pub mod repeated_enum;
