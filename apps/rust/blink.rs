static mut count: usize = 0;

fn timer_fired() {
    unsafe {
        count += 1;
        if count % 7 == 0 {
            super::println("Rust: Boop.");
        }
    }

    super::timer_subscribe(1 << 15, timer_fired);
    super::wait();
}

fn initialize() {
    super::println("I'm in the Rust app!");
    super::timer_subscribe(1 << 15, timer_fired);
    super::wait();
}

#[link_section = ".app.rust-boop"]
pub static RUST_BLINK_INIT: fn() = initialize;
