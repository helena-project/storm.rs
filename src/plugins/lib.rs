#![crate_name = "plugins"]
#![crate_type = "dylib"]
#![feature(plugin_registrar,quote,rustc_private,core,collections,std_misc)]

use rustc::plugin::Registry;

#[macro_use(span_note,span_err,__diagnostic_used)]
extern crate syntax;
extern crate rustc;

#[macro_use]
mod plugin_lib;
mod repeated_enum;
mod device_tree;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("repeated_enum", repeated_enum::expand);
    reg.register_macro("device_tree", device_tree::expand);
}
