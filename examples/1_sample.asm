; 1_sample.asm — first lesson: registers, MOV, INT 21h/02h.
;
; Demonstrates: MOV reg, imm / MOV AH, 2 / INT 21h/02h (print one char) / RET.
; v1 supports `MOV reg, imm` (no single-quote character literal) — we use the
; hex ASCII codes for printable chars.

org 100h

mov ah, 2
mov dl, 41h      ; 'A'
int 21h

mov dl, 42h      ; 'B'
int 21h

mov dl, 0Dh      ; CR
int 21h
mov dl, 0Ah      ; LF
int 21h

ret
