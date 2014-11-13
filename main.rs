#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![feature(macro_rules)]
#![feature(globs)]
#![feature(lang_items)]
#![feature(intrinsics)]

mod gpio;
mod ast;
mod nvic;
mod intrinsics;

#[lang="sized"]
pub trait Sized {}

#[lang="panic_bounds_check"]
fn fail_bounds_check(_: &(&'static str, uint),
                         _: uint, _: uint) -> ! {
    loop {}
}

#[lang="panic"]
fn panic(_: &(&'static str, &'static str, uint)) -> ! {
    loop {}
}

#[lang="sync"]
pub trait Sync {}

static LED : gpio::Pin = gpio::Pin { bus : gpio::PORT2, pin: 10 };

#[no_mangle]
pub extern fn AST_PER_Handler() {
    LED.toggle();
    let ast = unsafe { &mut *(ast::AST_BASE as u32 as *mut ast::Ast) };
    ast.start_periodic();
}

#[no_mangle]
pub extern fn main() -> int {
    LED.make_output();

    let ast = unsafe { &mut *(ast::AST_BASE as u32 as *mut ast::Ast) };
    ast.setup();
    ast.start_periodic();

    loop {
    }
}

