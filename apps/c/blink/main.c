#include <tock.h>

static uint32_t count = 0;

static void
timer_fired() {
  toggle_led();
  if (!(++count % 10)) {
    println("That's 10 timer fires.");
  }

  timer_subscribe(1 << 15, timer_fired);
  wait();
}

static void
init() {
  toggle_led();
  println("Welcome to the C blink app!");

  timer_subscribe(1 << 15, timer_fired);
  wait();
}

REGISTER_APP(blink, init);
