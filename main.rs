#![no_std]
#![no_main]
#![feature(macro_rules)]
#![feature(globs)]
#![feature(lang_items)]

mod gpio;

#[lang="sized"]
pub trait Sized {}

#[no_mangle]
pub extern fn main() -> int {
    let led0 = gpio::Pin {bus: gpio::PORT0, pin: 10};

    gpio::make_output(led0);
  
    loop {
        gpio::toggle(led0);
        let mut i = 0i;
        loop {
            i = i + 1;
            if i > 5000000 {
                break;  
            }
        }
    }
}

