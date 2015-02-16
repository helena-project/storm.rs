.cpu cortex-m4
.syntax unified
.thumb
.text

/* Exported functions */
.global SVC_Handler
.globl switch_to_user

/* SVC wrappers */
.globl __subscribe
.globl __command
.globl __wait

/* svc_rust_handler returns 0 to return to unprivileged stack (process) and 1
  to switch to privileged stack (kernel) */
.extern svc_rust_handler
.thumb_func
SVC_Handler:
  cmp lr, #0xfffffff9
  bne to_kernel
  movw lr, #0xfffd
  movt lr, #0xffff
  bx lr
to_kernel:
  mrs r0, PSP /* PSP into r0 */
  str r0, [sp, #0] /* PSP into Master stack r0 */
  movw LR, #0xFFF9
  movt LR, #0xFFFF
  bx lr


.thumb_func
/* r0 is start address and r1 is top of user stack */
switch_to_user:
    push {r4-r11}

    mov r4, r1
    /* Load the first stack frame */
    movw r5, #0      /* xPSR --> set thumb mode in EPSR (bit-24) */
    movt r5, #0x100
    stmdb r4!,  {r0, r5}  /* PC, xPSR */
    sub r4, r4, 24 /* Don't care about the next 6 registers */

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r4

    svc 0xff

    pop {r4-r11}
    bx lr

.thumb_func
__wait:
    svc 0
    bx lr

.thumb_func
__subscribe:
    svc 1
    bx lr

.thumb_func
__command:
    svc 2
    bx lr

