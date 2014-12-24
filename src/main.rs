#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(globs, asm, slicing_syntax)]

extern crate core;
extern crate drivers;
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
    use hal::usart::kstdio::*;
    use hal::gpio;
    use svc;

    static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

    static mut count : uint = 0;

    #[inline(never)]
    pub fn initialize() {
        kprint("I'm in the app!\n");
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
                kprint("Timer fired 10 times\n");
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

static mut PROCESS_STACK : [uint,..4096] = [0,..4096];

#[no_mangle]
pub extern fn main() -> int {
    use core::option::Option::*;
    use core::intrinsics::*;
    use hal::gpio::*;
    use hal::usart::kstdio::*;
    use hal::pm;
    use hal::pm::*;
    use hal::spi;
    use task;
    use task::Task::*;

    use drivers::flash_attr::FlashAttr;

    kstdio_init();

    {
        pm::enable_pba_clock(1); // SPI clock
        spi::set_mode(spi::MSTR::Master, spi::PS::Variable,
                      spi::RXFIFO::Disable, spi::MODFAULT::Disable);
        spi::enable();
        let mut flash_spi = spi::SPI {cs: 0};
        let mut flash_cs = Pin {bus: Port::PORT2, pin: 3};
        let mut miso = Pin {bus: Port::PORT2, pin: 4};
        let mut mosi = Pin {bus: Port::PORT2, pin: 5};
        let mut sclk = Pin {bus: Port::PORT2, pin: 6};
        let flash_attr = FlashAttr::initialize(&mut flash_spi, &mut flash_cs,
                                               &mut miso, &mut mosi, &mut sclk);

        if flash_attr.do_attr("welcome", |c| { kputc(c as char)}) {
            kputc('\n');
        } else {
            kprint("Welcome to the Tock OS!\n");
        }
    }

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

