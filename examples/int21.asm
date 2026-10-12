; int21.asm — show the four most common INT 21h sub-functions.
;
; 01h: read character (no echo)
; 02h: write character
; 09h: write $-terminated string
; 4Ch: exit program
;
; Demonstrates: MOV AH, n / INT 21h / DB string / RET
; v1 character literals use hex codes; 0Dh = CR, 0Ah = LF.

org 100h

mov ah, 9
mov dx, prompt
int 21h

mov ah, 1
int 21h          ; AL = character typed
mov bl, al       ; save it

mov ah, 2
mov dl, 0Dh
int 21h
mov dl, 0Ah
int 21h

mov ah, 9
mov dx, you_typed
int 21h

mov ah, 2
mov dl, bl
int 21h

mov dl, 0Dh
int 21h
mov dl, 0Ah
int 21h

mov ax, 4C00h
int 21h

prompt    db "type one char: $"
you_typed db "you typed: $"
