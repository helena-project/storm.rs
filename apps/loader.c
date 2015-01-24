#include <stdio.h>
#include <stdint.h>

extern uint32_t *apps_start;
extern uint32_t *apps_end;

int
main(int argc, char *argv[]) {
  for (uint32_t *app = apps_start; app != apps_end; app++) {
    *app = 0;
  }

  return 0;
}
