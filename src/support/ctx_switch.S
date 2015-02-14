.cpu cortex-m4
.syntax unified
.thumb
.text

/*; Exported functions*/
.global SVC_Handler
.global PendSV_Handler

.globl __ctx_to_user
.globl __ctx_to_master
.globl __subscribe
.globl __command
.globl __wait

/* svc_rust_handler returns 0 to return to unprivileged stack (process) and 1
  to switch to privileged stack (kernel) */
.extern svc_rust_handler
.thumb_func
SVC_Handler:
/*  push {lr}
  mrs r0, PSP
  bl svc_rust_handler
  pop {lr}
  cmp r0, 0
  bne __ctx_to_master
  bx lr*/
  mrs r0, PSP
  ldr r1, [r0, #24]
  ldr r2, [r1, #-2]
  ldr r1, [r0, #0]
  and r2, r2, #0xff
  str r2, [sp, #0]
  str r1, [sp, #4]
  b __ctx_to_master


.thumb_func
PendSV_Handler:
    mov r1, #0x2
    msr CONTROL, r1
    movw LR, #0xFFFD
    movt LR, #0xFFFF
    bx lr


.thumb_func
/* r0 is start address and r1 is top of user stack */
__ctx_to_user:
    /* Load the first stack frame */
    movw r2, #0      /* xPSR --> set thumb mode in EPSR (bit-24) */
    movt r2, #0x100
    stmdb r1!,  {r0, r2}  /* PC, xPSR */
    sub r1, r1, 24 /* Don't care about the next 6 registers */

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r1

    /* let r0 = &ICSR */
    movw r0, 0xED04
    movt r0, 0xE000

    /* *r0 = *r0 | 1 << 28 */
    ldr r1, [r0, #0]
    orr r1, 0x10000000
    str r1, [r0, #0]

    bx lr

.thumb_func
__ctx_to_master:
    mov r1, #0x0
    msr CONTROL, r1
    movw LR, #0xFFF9
    movt LR, #0xFFFF
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

