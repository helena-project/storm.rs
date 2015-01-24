#include <tock.h>
#include <commands.h>

void println(const char const *str) {
  for (const char const *c = str; *c != '\0'; c++) {
    __command(CMD_PRINTC, (uint32_t) *c, 0);
  }

  __command(CMD_PRINTC, (uint32_t) '\n', 0);
}

void toggle_led() {
  __command(CMD_TOGGLE_LED, 0, 0);
}

void timer_subscribe(uint32_t time, void (*f)(void)) {
  __subscribe(SUB_TIMER, time, (uint32_t) f);
}

void wait() {
  __wait(0, 0, 0);
}
