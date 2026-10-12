; sub_test.asm — subtract 16-bit numbers.
;
; Demonstrates: MOV reg, imm / SUB reg, imm / CMP / JNE / RET

org 100h

mov ax, 100
sub ax, 25       ; AX = 75

cmp ax, 75
jne bad

mov ah, 9
mov dx, ok_msg
int 21h
jmp done

bad:
mov ah, 9
mov dx, err_msg
int 21h

done:
ret

ok_msg  db "sub test passed$"
err_msg db "sub test failed$"
