; count_chars.asm — count characters in a $-terminated string.
;
; Demonstrates: MOV reg, imm / INC / CMP / JE / JMP / RET / DB.
; v1 supports `[SI]` as a memory pointer and `CMP reg16, imm16` directly;
; `CMP reg8, imm8` is in the milestone-2 ALU matrix.

org 100h

mov si, text      ; label reference → address of `text`
mov cx, 0

next_char:
    mov al, [si]
    mov ah, 0
    cmp ax, '$'    ; 16-bit compare of AX against ASCII `$`
    je  done
    inc cx
    inc si
    jmp next_char

done:
    mov ah, 2
    mov dl, cl         ; print low byte of count
    int 21h

    ret

text db "hello$"
