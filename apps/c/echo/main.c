#include <commands.h>
#include <tock.h>

static void
echo_byte() {
  char c;
  if (c = getchar()) {
    print("New char: ");
    __command(CMD_PRINTC, c, 0);
  }

  timer_subscribe(1 << 15, echo_byte);
  wait();
}

static void
init() {
  timer_subscribe(1 << 15, echo_byte);
  wait();
}

REGISTER_APP(echo, echo_byte);
