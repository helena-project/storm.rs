#ifndef COMMANDS_H
#define COMMANDS_H

#include <stdint.h>

void toggle_led();
void println(const char const *str);
void timer_subscribe(uint32_t time, void (*f)(void));
void wait();

#endif
