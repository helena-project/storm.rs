#![no_std]
#![no_main]
#![feature(macro_rules)]
#![feature(globs)]
#![feature(lang_items)]

mod gpio;
mod ast;

#[lang="sized"]
pub trait Sized {}

const LED0 : gpio::Pin = gpio::Pin {bus: gpio::PORT0, pin: 10};

#[no_mangle]
pub fn AST_OVF_Handler() {
    gpio::toggle(LED0);
}

#[no_mangle]
pub extern fn main() -> int {
    gpio::make_output(LED0);
    ast::setup();
    ast::start_periodic();

    loop {
    }
}

