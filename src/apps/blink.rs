static mut count: usize = 0;

#[inline(never)]
pub fn initialize() {
    super::toggle_led();
    super::writeln("I'm in the app!");

    unsafe {
        super::__subscribe(0, 1 << 15, timer_fired as usize);
        super::__wait();
    }
}

#[inline(never)]
pub fn timer_fired() {
    super::toggle_led();

    unsafe {
        count += 1;
        if count % 10 == 0 {
            super::writeln("Timer fired 10 times");
        }
    }

    unsafe {
        super::__subscribe(0, 1 << 15, timer_fired as usize);
        super::__wait();
    }
}
