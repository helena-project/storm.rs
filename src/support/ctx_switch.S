.cpu cortex-m4
.syntax unified
.thumb
.text

/*; Exported functions*/
.globl __prepare_user_stack
.globl __ctx_to_master
.globl __ctx_to_user
.globl __subscribe
.globl __command
.globl __wait

.thumb_func
/* r0 is start address and r1 is top of user stack */
__prepare_user_stack:
    /* Load the first stack frame */
    movw r2, #0      /* xPSR --> set thumb mode in EPSR (bit-24) */
    movt r2, #0x100
    stmdb r1!,  {r0, r2}  /* PC, xPSR */
    sub r1, r1, 24 /* Don't care about the next 6 registers */

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r1
    bx lr

.thumb_func
__ctx_to_user:
    mov r1, #0x2
    msr CONTROL, r1
    movw LR, #0xFFFD
    movt LR, #0xFFFF
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

