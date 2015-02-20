use commands::*;

static mut count: usize = 0;

fn timer_fired() {

    println("read TMP006 sensor");
    tmp006_read_sync();

    timer_subscribe(1 << 15, timer_fired);
    wait();
}

fn initialize() {
    println("Testing TMP006 sensor.");
    timer_subscribe(1 << 15, timer_fired);
    wait();
}

register_app!(".app.rust-test-tmp006", initialize);
