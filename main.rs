#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(macro_rules)]
#![feature(globs)]
#![feature(lang_items)]

mod gpio;
mod ast;

#[lang="sized"]
pub trait Sized {}

#[lang="fail_bounds_check"]
fn fail_bounds_check(_: &(&'static str, uint),
                         _: uint, _: uint) -> ! {
    loop {}
}

#[no_mangle]
pub extern fn AST_PER_Handler() {
    let led = gpio::Pin::new(gpio::PORT2, 10);
    led.toggle();
    let ast = unsafe { &mut *(ast::AST_BASE as u32 as *mut ast::Ast) };
    while ast.busy() {}
    ast.scr = 1 << 16;
}

#[no_mangle]
pub extern fn main() -> int {
    let led = gpio::Pin::new(gpio::PORT2, 10);
    led.make_output();

    let ast = unsafe { &mut *(ast::AST_BASE as u32 as *mut ast::Ast) };
    ast.setup();
    ast.start_periodic();

    loop {
    }
}

