.section .data

.equ CODE_SEG_SEL, 0x8
# fault handler descriptor
.equ INTR_GATE, 0x8e00
# trap handler descriptor
.equ TRAP_GATE, 0x8f00
# this size should be kept in sync with HLL code
.equ TASK_CPU_STATE_STRUCT_SIZE, 44

# interrupt vector definitions start
.align 8

idt_start:

# 0. divide error fault
.short isr0
.short CODE_SEG_SEL
.short INTR_GATE
.short 0x0

# 1. reserved handler descriptor
.short isr1
.short CODE_SEG_SEL
.short INTR_GATE
.short 0x0

# 2. NMI handler
.short isr2
.short CODE_SEG_SEL
.short INTR_GATE
.short 0x0

# 3. breakpoint trap
.short isr3
.short CODE_SEG_SEL
.short TRAP_GATE
.short 0x0

# 4. overflow trap
.short isr4
.short CODE_SEG_SEL
.short TRAP_GATE
.short 0x0

# filling standard protected mode inerrupt handlers
.irp n, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
.short isr\n
.short CODE_SEG_SEL
.short INTR_GATE
.short 0x0
.endr

# IRQ interrupts
.irp n, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47
.short isr\n
.short CODE_SEG_SEL
.short INTR_GATE
.short 0x0
.endr

# switch task syscall handler
.short switch_task
.short CODE_SEG_SEL
# privilege level should be fixed to make accessible from user space
.short INTR_GATE
.short 0x0

idt_end:

idt_limit = idt_end - idt_start - 1

idt_info:
.short idt_limit
.int idt_start

.global SCR_BUF
SCR_BUF:
.int 0xb8000

intr_handlers:
.rept 48
.int nop_intr_handler
.endr

.section .text

.macro create_stack_frame, bytes_for_locals=0

pushl %ebp
movl %esp, %ebp

subl $\bytes_for_locals, %esp

pushl %ebx
pushl %esi
pushl %edi

.endm

.macro destroy_stack_frame

popl %edi
popl %esi
popl %ebx

movl %ebp, %esp
popl %ebp

.endm

.macro ISR vecnum
isr\vecnum:

# saving registers
pushl %eax
pushl %ecx
pushl %edx

# interrupt handler table index
movl $\vecnum, %eax
# interrupt handler call
call *intr_handlers(, %eax, 4)

# restoring registers
popl %edx
popl %ecx
popl %eax

iret
.endm

# interrupt service routine for faults with error code
.macro ISRE vecnum
isr\vecnum:

# saving registers
pushl %eax
pushl %ecx
pushl %edx

# placing error code copy on the top of the stack
pushl 12(%esp)

# interrupt handler table index
movl $\vecnum, %eax
# interrupt handler call
call *intr_handlers(, %eax, 4)

# removing error code copy from the stack
addl $4, %esp

# restoring registers
popl %edx
popl %ecx
popl %eax

# removing error code from the stack
addl $4, %esp

iret
.endm

.global get_sp
get_sp:
movl %esp, %eax
addl $4, %eax
ret

.global get_eflags
get_eflags:
pushfl
popl %eax
ret

.global get_cs
get_cs:
movl $0, %eax
movw %cs, %ax
ret

.global register_handler
register_handler:

# interrupt vector number
movl 4(%esp), %edx
# pointer to interrupt handler function
movl 8(%esp), %eax

movl %eax, intr_handlers(, %edx, 4)

ret

# above function copy for HLL type safety
.global register_handler_with_err_code
register_handler_with_err_code:

# interrupt vector number
movl 4(%esp), %edx
# pointer to interrupt handler function
movl 8(%esp), %eax

movl %eax, intr_handlers(, %edx, 4)

ret

.global out_byte
out_byte:

create_stack_frame

# port number parameter
movl 12(%ebp), %edx

# byte to write stored as double word
movl 8(%ebp), %eax

# writing byte to specified I/O port
outb %al, %dx

destroy_stack_frame

ret

.global in_byte
in_byte:

create_stack_frame

# port number parameter
movl 8(%ebp), %edx

movl $0, %eax
inb %dx, %al

destroy_stack_frame

ret

.global load_idt
load_idt:

lidt idt_info

ret

.global intr_enable
intr_enable:

sti

ret

# default interrupt handling fuction
nop_intr_handler:

ret

.global syscall
syscall:

int $48

ret

# interrupt service routine gates
ISR 0
ISR 1
ISR 2
ISR 3
ISR 4
ISR 5
ISR 6
ISR 7
ISRE 8
ISR 9
ISRE 10
ISRE 11
ISRE 12
ISRE 13
ISRE 14
ISR 15
ISR 16
ISR 17
ISR 18
ISR 19
ISR 20
ISR 21
ISR 22
ISR 23
ISR 24
ISR 25
ISR 26
ISR 27
ISR 28
ISR 29
ISR 30
ISR 31

ISR 32
ISR 33
ISR 34
ISR 35
ISR 36
ISR 37
ISR 38
ISR 39
ISR 40
ISR 41
ISR 42
ISR 43
ISR 44
ISR 45
ISR 46
ISR 47

switch_task:

# saving cpu state
pushal

# passing stack pointer with saved current task state
pushl %esp
call save_current_task_state
addl $4, %esp

# switch to new task and get new stack pointer prepared for restore
# moved 32 bytes down
call switch_task_and_get_new_stack_ptr

# restoring stack
movl %eax, %esp

# restoring task we switched to state on the stack
pushl %esp
call restore_current_task_state
addl $4, %esp

# call print_task_state

# restoring registers
popal

# switching to new task completely
iret
