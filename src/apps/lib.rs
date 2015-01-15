#![crate_name = "apps"]
#![crate_type = "rlib"]
#![no_std]

extern crate platform;
extern crate hil;

#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num : usize, arg1 : usize, arg2 : usize) -> isize;
    fn __command(driver_num : usize, arg1 : usize, arg2 : usize) -> isize;
    fn __wait() -> isize;
}

pub mod blinkapp {
    use platform::sam4l::usart::kstdio::*;
    use platform::sam4l::gpio;
    use hil::gpio::*;

    static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

    static mut count : usize = 0;

    #[inline(never)]
    pub fn initialize() {
        LED.make_output();
        kprint("I'm in the app!\n");

        unsafe {
            super::__subscribe(0, 1 << 15, timer_fired as usize);
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
            super::__subscribe(0, 1 << 15, timer_fired as usize);
            super::__wait();
        }
    }
}

