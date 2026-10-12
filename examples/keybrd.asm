; keybrd.asm — read a keystroke via INT 16h.
;
; INT 16h AH=00h: wait for a key, return AL = ASCII char, AH = scan code.
;
; Demonstrates: INT 16h / MOV AH, 2 / INT 21h/02h / RET

org 100h

mov ah, 0
int 16h          ; AL = ASCII char

mov ah, 2
mov dl, al
int 21h

mov dl, 0Dh
int 21h
mov dl, 0Ah
int 21h

ret
