use commands::*;

static mut count: usize = 0;

fn timer_fired() {

    println("read TMP006 sensor");
    tmp006_read_sync();

    timer_subscribe(1 << 15, timer_fired);
}

fn initialize() {
    println("Testing TMP006 sensor.");
    timer_subscribe(1 << 15, timer_fired);
    loop {
        wait();
    }
}

// The I2C on both the Firestomrs 1.1 and 1.3 is busted (respectively, the temp sensor and light
// sensor). Uncomment this line if you have a functioning version.
//register_app!(".app.rust-test-tmp006", initialize);
