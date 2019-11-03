.section .data

# GDT

# null segment descriptor
.fill 8

# system level code segment descriptor
sys_code_seg:
# low bits of limit
.short 0xffff

# low 24 bit of base
.short 0
.byte 0

# access rights byte 10011000
# code segment with only-execute rights
.byte 0x98

# granularity and limit 11001111
.byte 0xcf

# high byte of base
.byte 0

# system level data segment descriptor
data_code_seg:
# limit
.short 0xffff
# base
.short 0
.byte 0

# data segment with R/W rights 10010010
.byte 0x92
# granularity and limit
.byte 0xcf

# base high byte
.byte 0

.section .text

# stack and data segment selection
movw $0x10, %ax

movw %ax, %ds
movw %ax, %ss
movw %ax, %es
movw %ax, %gs
movw %ax, %fs

movl $STACK_TOP, %esp

# calling VM initialization function before 
# transferring control to system
call init_vm

# enabling paging by setting PG bit
movl $0x7fffffff, %ecx
addl $1, %ecx
movl %cr0, %eax
orl %ecx, %eax
movl %eax, %cr0

xorl %eax, %eax
movb 0xa001, %al
subl $40, %eax
movb %al, 0xb80a0

movb 0x2001, %al
subl $40, %eax
mov %al, 0xb80a2

# jumping to system code
pushl $0x9220
ret

.global set_cr3
set_cr3:

movl 4(%esp), %eax
movl %eax, %cr3

ret
