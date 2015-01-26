#include <stdint.h>

extern uint32_t *_sapps;
extern uint32_t *_eapps;

typedef void (*app_init)(void);
static uint32_t STACK[4096];

extern void switch_to_user(uint32_t *pc, uint32_t *stack);

void
__start_apps() {
  switch_to_user(_sapps, STACK);
  ((app_init)_sapps)();
  /* for (uint32_t *app = _sapps; app != _eapps; app++) { */
  /* } */
}
