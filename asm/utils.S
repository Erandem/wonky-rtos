.equ __SP_L__, 0x3D
.equ __SP_H__, 0x3E
.equ __SREG__, 0x3F

.global utils_get_stack_pointer
.global utils_get_sreg

utils_get_stack_pointer:
    in r24, __SP_L__
    in r25, __SP_H__
    ret


utils_get_sreg:
    in r24, __SREG__
    ret