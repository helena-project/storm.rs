#ifndef TOCK_H
#define TOCK_H

#include <stdint.h>

#define SVC_ROUTINE(name, number) \
  static inline void __##name(uint32_t a, uint32_t b, uint32_t c) {\
    register uint32_t _a asm("r0") = a;               \
    register uint32_t _b asm("r1") = b;               \
    register uint32_t _c asm("r2") = c;               \
    asm volatile(                                     \
        "svc " #number "\n\t"                         \
        :: "r" (_a), "r" (_b), "r" (_c)               \
    );                                                \
  }

SVC_ROUTINE(wait, 0)
SVC_ROUTINE(subscribe, 1)
SVC_ROUTINE(command, 2)

// List of commands
#define CMD_PRINTC 0
#define CMD_TOGGLE_LED 1

// List of subscriptions
#define SUB_TIMER 0

#endif
