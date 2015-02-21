#include <commands.h>
#include <tock.h>

static void
echo_byte(uint8_t byte) {
  /* print("New char: "); */
  /* __command(CMD_PRINTC, byte, 0); */
}

static void
init() {
  println("Hello, from C echo.");
  readc_subscribe(echo_byte);
  wait();
}

//REGISTER_APP(echo, init);
