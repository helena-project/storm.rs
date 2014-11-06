#![no_std]
#![no_main]
#![feature(macro_rules)]
#![feature(globs)]
#![feature(lang_items)]

mod gpio;
mod ast;

#[lang="sized"]
pub trait Sized {}


#[no_mangle]
pub extern fn AST_OVF_Handler() {
    let led = gpio::Pin::new(gpio::PORT2, 10);
    led.toggle();
}

#[no_mangle]
pub extern fn main() -> int {
    let led = gpio::Pin::new(gpio::PORT2, 10);
    led.make_output();
    led.toggle();

    ast::setup();
    ast::start_periodic();

    loop {
    }
}

