use commands::*;

static mut count: usize = 0;

fn timer_fired() {
    unsafe {
        count += 1;
        if count % 7 == 0 {
            println("Rust: Boop.");
        }
    }

    timer_subscribe(1 << 15, timer_fired);
    wait();
}

fn initialize() {
    println("I'm in the Rust app!");
    timer_subscribe(1 << 15, timer_fired);
    wait();
}

register_app!(".app.rust-boop", initialize);
