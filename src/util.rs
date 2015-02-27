use core::prelude::*;
use config;

pub fn println(val: &str) {
    let mut console = unsafe {
        config::Console.as_mut().expect("Console is None!")
    };

    console.writeln(val);
}

// gratefully borrowed from
//  http://www.sparetimelabs.com/tinyprintf/tinyprintf.php
pub fn print_num(val: u32) {
    let mut console = unsafe {
        config::Console.as_mut().expect("Console is None!")
    };

    let mut num = val;
    let mut first = true;
    let mut d = 1;
    let base = 10;

    while num/d >= base {
        d *= base;
    }

    while d != 0 {
        let digit = num / d;
        num %= d;
        d /= base;

        if !first || digit > 0 || d==0 {
            console.putc((digit + 0x30) as u8);
            first = false;
        }
    }

    console.putc('\n' as u8);
}

