#include <commands.h>

#define REGISTER_APP(name, init) \
  void (*name)() __attribute__((section(".app.blink"))) = init

static uint32_t count = 0;

void
timer_fired() {
  if (!(++count % 10)) {
    println("That's 10 timer fires.");
  }

  timer_subscribe(1 << 15, timer_fired);
  wait();
}

void
init() {
  toggle_led();
  println("Welcome to the blink app!");

  timer_subscribe(1 << 15, timer_fired);
  wait();
}

REGISTER_APP(blink, init);
