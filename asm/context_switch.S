.equ __SP_L__, 0x3D
.equ __SP_H__, 0x3E
.equ __SREG__, 0x3F

.extern ksched
.extern panic_asm
.extern ASM_LOG_LOC

.extern kernel_scheduler_tick

.global save_context
.global exec_context

; Saves the registers and returns the stack pointer as u16
save_context:
    ; Push remaining registers
    push r0
    push r1
    push r2
    push r3
    push r4
    push r5
    push r6
    push r7
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    push r16
    push r17
    push r18
    push r19
    push r20
    push r21
    push r22
    push r23
    push r24
    push r25
    push r26
    push r27
    push r28
    push r29
    push r30
    push r31

    in r0, __SREG__
    push r0

    in r24, __SP_L__
    in r25, __SP_H__

    ; We can't return here - we've mangled the return address, which is at the
    ; beginning of the stack. We need to change this to an interrupt handler.
    jmp kernel_scheduler_tick

exec_context:
    ; r24 - low byte of stack pointer
    ; r25 - high byte of stack pointer

    ; load the passed stack pointer
    out __SP_L__, r24
    out __SP_H__, r25

    ; restore SREG
    pop r0
    out __SREG__, r0

    ; restore registers in descending order
    pop r31
    pop r30
    pop r29
    pop r28
    pop r27
    pop r26
    pop r25
    pop r24
    pop r23
    pop r22
    pop r21
    pop r20
    pop r19
    pop r18
    pop r17
    pop r16
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop r7
    pop r6
    pop r5
    pop r4
    pop r3
    pop r2
    pop r1
    pop r0

    ; return to the passed stack frame
    ret