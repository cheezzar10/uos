[bits 16]
[org 7c3eh]

[section .text]

; opening A20 line to make all RAM accessible (disable 1MB wraparound)
in al, 92h
or al, 2
out 92h, al

mov di, 0

%macro read_sectors 5

; head number
push %5
; how many sectors to read
push %4
; MBR is the first sector - skipping 
; start sector
push %3
; track number
push %2
; read buffer address ( location is ES:<buf addr> )
push %1

call read

; removing arguments from the stack
add sp, 10

%endmacro

mov ax, 0
mov es, ax

; loading 2nd stage loader ( assuming it's not larger than 4k )
; read(char* buffer, size_t track_num, size_t start_sector, size_t sector_count, size_t head_num)
read_sectors 0f000h, 0, 2, 8, 0

; moving system binary image load buffer segment register at 64k
mov ax, 1000h
mov es, ax

; reading system binary image sectors
read_sectors 0h, 0, 1, 18, 1

read_sectors 2400h, 1, 1, 18, 0

read_sectors 4800h, 1, 1, 18, 1

read_sectors 6c00h, 2, 1, 18, 0

read_sectors 9000h, 2, 1, 18, 1

read_sectors 0b400h, 3, 1, 18, 0

; jumping to second stage loader
jmp loader_jmp

; read(char* buffer, size_t track_num, size_t start_sector, size_t sector_count, size_t head_num)
read:
; performing stack frame setup
push bp
mov bp, sp

; allocate stack space for locals here if needed

; saving callee safe registers
push bx
push si
push di

; load specified number of sectors to specified buffer
; load sectors BIOS service
mov ah, 02h
; using sector_count parameter
mov al, byte [bp+10]
; read buffer address parameter
mov bx, [bp+4]

; starting sector number parameter
mov cl, byte [bp+8]
; track number 0 
mov ch, byte [bp+6]
; using head number parameter
mov dh, byte [bp+12]
; using drive number 0
mov dl, 0

; performing call to BIOS service
int 13h

; restoring callee safe registers
pop di
pop si
pop bx

; destroying stack frame
mov sp, bp
pop bp

; returning
ret

; here long jump to boot loader should be encoded directly in data section

loader_jmp:

cli

; loading global memory segment descriptors table
lgdt [gdt_info]

; activating protected mode
mov eax, cr0
or eax, 1
mov cr0, eax

jmp loader_long_jump

[section .data]
loader_long_jump:
; long jump op code
db 0eah
; 2nd stage loader init code location
dw 0f200h
; system code segment selector
dw 08h

gdt_info:
dw 17h
dd 0f000h
