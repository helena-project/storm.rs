#![crate_name = "apps"]
#![crate_type = "rlib"]
#![feature(asm, lang_items, globs)]
#![feature(phase)]
#![no_std]

extern crate hal;
extern crate hil;

#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num : uint, arg1 : uint, arg2 : uint) -> int;
    fn __command(driver_num : uint, arg1 : uint, arg2 : uint) -> int;
    fn __wait() -> int;
}

pub mod blinkapp {
    use hal::usart::kstdio::*;
    use hal::gpio;
    use hil::gpio::*;

    static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

    static mut count : uint = 0;

    #[inline(never)]
    pub fn initialize() {
        LED.make_output();
        kprint("I'm in the app!\n");

        unsafe {
            super::__subscribe(0, 1 << 15, timer_fired as uint);
            super::__wait();
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

        unsafe {
            super::__subscribe(0, 1 << 15, timer_fired as uint);
            super::__wait();
        }
    }
}

