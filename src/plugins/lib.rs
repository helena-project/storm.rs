#![crate_name = "plugins"]
#![crate_type = "dylib"]
#![feature(plugin_registrar,quote,rustc_private,core,collections,std_misc)]

use rustc::plugin::Registry;

#[macro_use(span_note,span_err)]
extern crate syntax;
extern crate rustc;

#[macro_use]
mod plugin_utils;
mod tree_plugin_utils;
mod repeated_enum;
mod device_tree;
mod platform_tree;
mod config_tree;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("repeated_enum", repeated_enum::expand);
    reg.register_macro("device_tree", device_tree::expand);
    reg.register_macro("platform_tree", platform_tree::expand);
    reg.register_macro("config_tree", config_tree::expand);
}
