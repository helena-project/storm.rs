#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs, asm)]

extern crate core;
extern crate hal;
extern crate support;

#[allow(improper_ctypes)]
extern {
    fn __prepare_user_stack(start : uint, user_stack : *mut uint);
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
    pub const YIELD : u16 = 0;
    pub const ADD_TIMER : u16 = 1;
}

mod app {
    use hal::usart;
    use hal::gpio;
    use svc;

    static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

    static mut count : uint = 0;

    #[inline(never)]
    pub fn initialize() {
        let uart = usart::USART::UART3;
        uart.print("I'm in the app!\n");
        LED.make_output();

        let ts : u32 = 1 << 15;
        unsafe {
            asm!("mov r0, $0; mov r1, $1; svc $2"
                    :
                    :"r"(ts),"r"(timer_fired),"i"(svc::ADD_TIMER)
                    :"r0"
                    : "volatile");
        }
        unsafe {
            asm!("svc $0" ::"i"(svc::YIELD):: "volatile");
        }
    }

    #[inline(never)]
    pub fn timer_fired() {
        LED.toggle();

        unsafe {
            count = count + 1;
            if count % 10 == 0 {
                let uart = usart::USART::UART3;
                uart.print("Timer fired 10 times\n");
            }
        }

        let ts : u32 = 1 << 15;
        unsafe {
            asm!("mov r0, $0; mov r1, $1; svc $2"
                    :
                    :"r"(ts),"r"(timer_fired),"i"(svc::ADD_TIMER)
                    :"r0"
                    : "volatile");
        }
        unsafe {
            asm!("svc $0" ::"i"(svc::YIELD):: "volatile");
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
    use task;
    use task::Task::*;

    let uart = usart::USART::UART3;
    Pin {bus : Port::PORT1, pin : 9}.set_peripheral_function(PeripheralFunction::A);
    Pin {bus : Port::PORT1, pin : 10}.set_peripheral_function(PeripheralFunction::A);

    pm::enable_pba_clock(11); // USART3 clock
    uart.init_uart();
    uart.set_baud_rate(115200);
    uart.enable_tx();

    uart.print("Starting tock...\n");
    timer::setup();

    unsafe {
        task::setup();
    }
    task::Task::UserTask(app::initialize as uint).post();

    loop {
        match unsafe { task::dequeue() } {
            None => {
                support::wfi(); // Sleep!
            },
            Some(task) => {
                match task {
                    UserTask(task_addr) => unsafe {
                        __prepare_user_stack(task_addr,
                            &mut PROCESS_STACK[255]);
                        let icsr : *mut uint = 0xE000ED04 as *mut uint;
                        volatile_store(icsr,
                            volatile_load(icsr as *const uint) | 1<<28);
                    },
                    KernelTask(task) => {
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
pub unsafe extern fn SVC_Handler(r0 : uint, r1 : uint) {
    use core::intrinsics::volatile_load;

    let mut psp : uint = 0;
    asm!("mrs $0, PSP" :"=r"(psp)::: "volatile");

    /* Find process PC on stack */
    let user_pc = volatile_load((psp + 24) as *const uint);

    /* SVC is one instruction before current PC. The low byte is the opcode */
    let svc = volatile_load((user_pc - 2) as *const u16) & 0xff;
    match svc {
        svc::YIELD => {},
        svc::ADD_TIMER => {
            let alarm_task = task::Task::UserTask(r1);
            timer::set_alarm(r0 as u32, alarm_task);
            return ();
        },
        _ => {}
    }

    __ctx_to_master();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn PendSV_Handler() {
    __ctx_to_user();
}

