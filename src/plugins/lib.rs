#![crate_name = "plugins"]
#![crate_type = "dylib"]
#![feature(plugin_registrar,quote,rustc_private,core,collections)]

extern crate syntax;
extern crate rustc;

pub mod repeated_enum;
