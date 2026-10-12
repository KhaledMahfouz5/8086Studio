; sum.asm — add two 16-bit numbers and exit with the low byte as the code.
;
; Demonstrates: MOV reg, imm / ADD reg, imm / MOV AX, 4C00h / INT 21h/4Ch

org 100h

mov ax, 1234h
mov bx, 5678h
add ax, bx       ; AX = 68ACh

mov ah, 4Ch      ; exit with code = AL = low byte of sum
int 21h
