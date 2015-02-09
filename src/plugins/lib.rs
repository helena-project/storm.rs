#![crate_name = "plugins"]
#![crate_type = "dylib"]
#![feature(plugin_registrar,quote,rustc_private,core,collections)]

use rustc::plugin::Registry;

#[macro_use(span_note,span_err,__diagnostic_used)]
extern crate syntax;
extern crate rustc;

macro_rules! parse_int_lit {
    ($parser:expr, $cx:expr, $sp:expr) => (
        match $parser.parse_lit().node {
            Lit_::LitInt(n, _) => n,
            _ => {
                ($cx).span_err(($parser).last_span,
                    "Expected an integer literal.");
                0
            }
        }
    );
}

mod repeated_enum;
mod device_tree;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("repeated_enum", repeated_enum::expand);
    reg.register_macro("device_tree", device_tree::expand);
}
