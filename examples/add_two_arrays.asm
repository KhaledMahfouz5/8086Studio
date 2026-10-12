; add_two_arrays.asm — sum two byte arrays element-wise.
;
; Demonstrates: MOV reg16,[disp16] / MOV reg8,[disp16] / ADD / DB / labels.
; Each byte pair (a[i], b[i]) is summed; because every result fits in a byte
; we can add two adjacent bytes as one 16-bit word and store it back.

org 100h

; elements 0 and 1, summed as one word
mov ax, [a]
mov bx, [b]
add ax, bx
mov [c], ax

; elements 2 and 3
mov ax, [a+2]
mov bx, [b+2]
add ax, bx
mov [c+2], ax

; element 4
mov al, [a+4]
mov ah, 0
mov bl, [b+4]
mov bh, 0
add ax, bx
mov [c+4], al

ret

a db 1, 2, 3, 4, 5
b db 10, 20, 30, 40, 50
c db 0, 0, 0, 0, 0
