#ifndef COMMANDS_H
#define COMMANDS_H

#include <stdint.h>

void toggle_led();
void print(const char const *str);
void println(const char const *str);
uint8_t getchar();

void timer_subscribe(uint32_t time, void (*f)(void));
void readc_subscribe(void (*f)(uint8_t));


/* the C wait implementation doesn't work for some reason (gcc stacks r7 again,
 * which seems to break popping the stack, even though it really shouldn't...).
 * For now, use the assembly version in src/support/ctx_switch.S
 * void wait();
 */

void __wait();
#define wait __wait

#endif
