#include <stdint.h>

extern uint32_t _sapps[];
extern uint32_t _eapps[];

typedef void (*app_init)(void);
static uint32_t STACK[16][256] = {0};

extern void switch_to_user(uint32_t pc, uint32_t *stack);

void
__start_apps() {
  uint32_t num_apps = _eapps - _sapps;
  for (uint32_t i = 0; i < num_apps; ++i) {
    switch_to_user(_sapps[i], STACK[i + 1]);
  }
}
