.section .data

.global SCREEN_BUF
SCREEN_BUF:
.int 0xb8000

.section .text

.global get_sp
get_sp:
movl %esp, %eax
addl $4, %eax
ret

