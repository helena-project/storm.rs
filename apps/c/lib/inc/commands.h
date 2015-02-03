#ifndef COMMANDS_H
#define COMMANDS_H

#include <stdint.h>

#define REGISTER_APP(name, init) \
  void (*name)() __attribute__((section(".app." #name))) = init

void toggle_led();
void print(const char const *str);
void println(const char const *str);
uint8_t getchar();

void timer_subscribe(uint32_t time, void (*f)(void));
void readc_subscribe(void (*f)(uint8_t));
void wait();

#endif
