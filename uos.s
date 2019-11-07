.section .data

.global SCREEN_BUF
SCREEN_BUF:
.int 0xb8000

.global get_sp
get_sp:
movl %esp, %eax
ret

.global memcpy
memcpy:
# stack frame creation
pushl %ebp
movl %esp, %ebp

# saving registers
pushl %esi
pushl %edi

# bytes counter
movl 16(%ebp), %ecx
# source addr
movl 12(%ebp), %esi
# destination addr
movl 8(%ebp), %edi

# clearing direction flag
cld

# performing copy
rep movsb

# should return pointer to destination buffer
movl %edi, %eax

# restoring registers
popl %edi
popl %esi

# stack frame cleanup
leave

ret

.global memset
memset:
# stack frame setup
pushl %ebp
movl %esp, %ebp

# saving registers
pushl %edi

# bytes counter
movl 16(%ebp), %ecx
# source addr
movl 12(%ebp), %eax
# destination addr
movl 8(%ebp), %edi

# initializing bytes
rep stosb
# returns ptr to destination buf
movl %edi, %eax

# restoring registers
popl %edi

# stack frame cleanup
leave

ret

.global memcmp
memcmp:

pushl %ebp
movl %esp, %ebp

pushl %esi
pushl %edi

movl 16(%ebp), %ecx
movl 12(%ebp), %edi
movl 8(%ebp), %esi

cld
repe cmpsb

je equal

jg greater

movl $-1, %eax
jmp return

greater:
movl $1, %eax
jmp return

equal:
movl $0, %eax

return:

popl %edi
popl %esi

leave

ret

.global bcmp
bcmp:

# delegating to memcpy
call memcmp

ret
