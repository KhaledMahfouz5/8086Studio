; loops.asm — three kinds of loop: LOOP / CMP+Jcc.
;
; Demonstrates: MOV CX, n / LOOP / INC / CMP / JE / JNE / RET

org 100h

; 1) simple loop: count down CX from 5 to 1, print "A" each iteration
mov cx, 5
top1:
    mov ah, 2
    mov dl, 41h      ; 'A'
    int 21h
    loop top1

mov dl, 0Dh
int 21h
mov dl, 0Ah
int 21h

; 2) cmp+jne: print "1" through "5"
mov bx, 1
top2:
    mov ah, 2
    mov dx, bx       ; v1 ALU supports reg16,imm16 directly
    add dx, 2Fh      ; '0' - 1 = 0x2F
    int 21h

    inc bx
    cmp bx, 5
    jne top2

ret
