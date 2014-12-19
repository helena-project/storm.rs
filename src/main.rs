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

mod std {
    pub use core::*;
}

mod task;
mod timer;
mod ringbuf;

mod svc {
    pub const TEST : u16 = 72;
    pub const ADD_TIMER : u16 = 1;
}

mod app {
    use hal::usart;
    use hal::gpio;
    use svc;

    static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

    #[inline(never)]
    pub fn initialize() {
        let uart = usart::USART::UART3;
        uart.print("I'm in the app!\n");
        LED.make_output();

        unsafe {
            asm!("svc $0" ::"i"(svc::ADD_TIMER):: "volatile");
        }
    }

    #[inline(never)]
    pub fn timer_fired() {
        let uart = usart::USART::UART3;
        uart.print("Timer fired\n");
        LED.toggle();

        unsafe {
            asm!("svc $0" ::"i"(svc::ADD_TIMER):: "volatile");
        }
    }
}

static mut PROCESS_STACK : [uint,..256] = [0,..256];

#[no_mangle]
pub extern fn main() -> int {
    use core::option::Option::*;
    use core::intrinsics::*;
    use hal::gpio::*;
    use hal::usart;
    use hal::pm;
    use hal::pm::*;

    let uart = usart::USART::UART3;
    Pin {bus : Port::PORT1, pin : 9}.set_peripheral_function(PeripheralFunction::A);
    Pin {bus : Port::PORT1, pin : 10}.set_peripheral_function(PeripheralFunction::A);

    pm::enable_pba_clock(11);
    uart.init_uart();
    uart.set_baud_rate(115200);
    uart.enable_tx();

    uart.print("Starting tock...\n");
    timer::setup();

    unsafe {
        task::setup();
    }
    task::Task{f:app::initialize, user:true}.post();

    loop {
        loop {
            match unsafe { task::dequeue() } {
                None => {
                    uart.print("Going to sleep\n");
                    support::wfi(); // Sleep!
                    uart.print("Awake!\n");
                },
                Some(task::Task{f: task, user: u}) => {
                    if u {
                        unsafe {
                            __prepare_user_stack(task,
                                &mut PROCESS_STACK[255]);
                          let icsr : *mut uint = 0xE000ED04 as *mut uint;
                          volatile_store(icsr,
                              volatile_load(icsr as *const uint) | 1<<28);
                        }
                    } else {
                        task();
                    }
                }
            }
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[allow(unused_assignments)]
pub unsafe extern fn SVC_Handler() {
    use core::intrinsics::volatile_load;
    use hal::usart;

    let uart = usart::USART::UART3;
    uart.print("In SVC Handler!\n");

    let mut psp : uint = 0;
    asm!("mrs $0, PSP" :"=r"(psp)::: "volatile");

    /* Find process PC on stack */
    let user_pc = volatile_load((psp + 24) as *const uint);

    /* SVC is one instruction before current PC. The low byte is the opcode */
    let svc = volatile_load((user_pc - 2) as *const u16) & 0xff;
    match svc {
        svc::TEST => uart.print("Success!\n"),
        svc::ADD_TIMER => {
            let alarm_task = task::Task{f:app::timer_fired, user: true};
            timer::set_alarm(1 << 16, alarm_task);
            uart.print("Add timer\n");

        },
        _ => uart.print("Bad SVC\n")
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

