#![no_std]
#![no_main]
#![feature(macro_rules)]
#![feature(globs)]
#![feature(lang_items)]

mod gpio;
mod ast;

#[lang="sized"]
pub trait Sized {}

const LED0 : gpio::Pin = gpio::Pin {bus: gpio::PORT2, pin: 10};

#[no_mangle]
pub extern fn AST_OVF_Handler() {
    LED0.toggle();
}

#[no_mangle]
pub extern fn main() -> int {
    LED0.make_output();
    LED0.toggle();

    ast::setup();
    ast::start_periodic();

    loop {
    }
}

