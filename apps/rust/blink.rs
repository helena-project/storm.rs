static mut count: usize = 0;

pub fn timer_fired() {
    super::toggle_led();

    unsafe {
        count += 1;
        if count % 10 == 0 {
            super::println("Timer fired 10 times");
        }
    }

    super::timer_subscribe(1 << 15, timer_fired);
    super::wait();
}

pub fn initialize() {
    super::toggle_led();
    super::println("I'm in the Rust app!");

    super::timer_subscribe(1 << 15, timer_fired);
    super::wait();
}

#[link_section = ".app.rust-blink"]
pub static RUST_BLINK_INIT: fn() = initialize;
