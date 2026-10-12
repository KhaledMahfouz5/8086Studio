; pushpop.asm — save and restore registers with PUSH/POP.
;
; Demonstrates: MOV / PUSH / POP / ADD / RET

org 100h

mov ax, 100
mov bx, 50

push ax         ; save 100
push bx         ; save 50

; modify them
mov ax, 1
mov bx, 2

; restore (note: POP reverses order)
pop bx          ; bx = 50 again
pop ax          ; ax = 100 again

; verify by adding them
add ax, bx      ; ax = 150

; print the high byte by exiting with code = AL = low byte
mov ah, 4Ch
int 21h
