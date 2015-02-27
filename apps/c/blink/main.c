#include <tock.h>

uint32_t count = 0;
uint32_t max = 10;

static void
timer_fired() {
  toggle_led();
  if (!(++count % max)) {
    println("That's 10 timer fires.");
  }

  timer_subscribe(1 << 15, timer_fired);
}

static void
init() {
  toggle_led();
  println("Welcome to the C blink app!");

  timer_subscribe(1 << 15, timer_fired);
  while(1) {
    wait();
  }
}

REGISTER_APP(blink, init);
