; hello.asm — the canonical hello-world.
; assemble, then Run. Output goes to the studio's "BIOS" status line.
;
; Demonstrates: MOV reg, imm / MOV reg, label / INT 21h AH=09h / RET / DB string

org 100h

mov dx, msg
mov ah, 9
int 21h
ret

msg db "hello, world$"
