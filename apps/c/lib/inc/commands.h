#ifndef COMMANDS_H
#define COMMANDS_H

#include <stdint.h>

#define REGISTER_APP(name, init) \
  void (*name)() __attribute__((section(".app." #name))) = init

void toggle_led();
void println(const char const *str);
void timer_subscribe(uint32_t time, void (*f)(void));
void wait();

#endif
