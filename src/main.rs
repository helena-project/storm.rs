#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs, asm)]

extern crate core;
extern crate hal;
extern crate support;

#[allow(improper_ctypes)]
extern {
    fn __prepare_user_stack(start : fn(), user_stack : *mut uint);
    fn __ctx_to_user();
    fn __ctx_to_master();
}

mod task;
mod timer;
mod init;
mod ringbuf;

mod svc {
    pub const TEST : u16 = 72;
}

#[inline(never)]
fn app() {
    use hal::usart;
    let uart = usart::USART::UART3;
    uart.print("I'm in the app!\n");
    unsafe {
        asm!("svc $0" ::"i"(svc::TEST):: "volatile");
    }
}

static mut PROCESS_STACK : [uint,..256] = [0,..256];

#[no_mangle]
pub extern fn main() -> int {
    use hal::gpio::*;
    use hal::usart;
    use hal::pm;

    let uart = usart::USART::UART3;
    Pin {bus : PORT1, pin : 9}.set_peripheral_function(A);
    Pin {bus : PORT1, pin : 10}.set_peripheral_function(A);

    pm::enable_pba_clock(11);
    uart.init_uart();
    uart.set_baud_rate(115200);
    uart.enable_tx();

    uart.print("Starting tock...\n");

    unsafe {
      __prepare_user_stack(app, &mut PROCESS_STACK[255]);
      let icsr : *mut uint = 0xE000ED04 as *mut uint;
      core::intrinsics::volatile_store(icsr,
        core::intrinsics::volatile_load(icsr as *const uint) | 1<<28);
    }

    loop {
        uart.print("Going to sleep\n");
        support::wfi(); // Sleep!
        uart.print("Awake!\n");
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[allow(unused_assignments)]
pub unsafe extern fn SVC_Handler() {
    use hal::usart;
    let uart = usart::USART::UART3;
    uart.print("In SVC Handler!\n");

    let mut psp : uint = 0;
    asm!("mrs $0, PSP" :"=r"(psp)::: "volatile");
    let user_pc = core::intrinsics::volatile_load((psp + 24) as *const uint);
    let svc = core::intrinsics::volatile_load((user_pc - 2) as *const u16) & 0xff;
    match svc {
        svc::TEST => uart.print("Success!\n"),
        _ => uart.print("Ooops...\n")
    }

    __ctx_to_master();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn PendSV_Handler() {
    use hal::usart;
    let uart = usart::USART::UART3;
    uart.print("In PendSV Handler!\n");
    __ctx_to_user();
}

