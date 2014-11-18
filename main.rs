#![no_main]
#![no_std]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![feature(macro_rules)]
#![feature(lang_items)]
#![feature(intrinsics)]
#![feature(asm)]

extern crate core;

mod gpio;
mod ast;
mod nvic;
mod task;
mod lang_items;
pub mod support;

static LED : gpio::Pin = gpio::Pin { bus : gpio::PORT2, pin: 10 };

#[no_mangle]
pub extern fn AST_PER_Handler() {
    task::post(toggle_led);
    let ast = unsafe { &mut *(ast::AST_BASE as u32 as *mut ast::Ast) };
    ast.start_periodic();
}

fn toggle_led() {
    LED.toggle();
}

fn app_entry() {
    LED.make_output();

    let ast = unsafe { &mut *(ast::AST_BASE as u32 as *mut ast::Ast) };
    ast.setup();
    ast.start_periodic();
}

#[no_mangle]
pub extern fn main() -> int {
    task::post(app_entry);
    loop {
      use core::option::{None, Some};
      match unsafe { task::dequeue() } {
        None => {
            support::wfi() // Sleep!
        },
        Some(task::Task(task)) => { task() }
      }
    }
}

