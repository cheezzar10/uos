[bits 16]
[org 7c3eh]

[section .text]

; clearing the screen using scroll up function 0x6

; scroll up (06) entire screen (00)
mov ax, 0600h
; setting normal text attribute (white text on black screen)
mov bh, 07h
; starting from row (0) column (0)
mov cx, 0h
; to row (24) column (79)
mov dx, 184fh
; calling BIOS video service
int 10h

; opening A20 line to make all RAM accessible (disable 1MB wraparound)
in al, 92h
or al, 2
out 92h, al

; loading 2nd stage loader and placing it at the address 0x8000

; reading all sectors on the FDD track minus MBR
push 17
; MBR is the first sector - skipping it
push 2
; reading from track 0
push 0
; loading directly to 2nd stage loader location in memory
push 8000h

; actual load buffer location is ES:<buf addr>
; initializing ES to zero

mov ax, 0
mov es, ax

call read

; removing arguments from the stack
add sp, 6

; jumping to second stage loader
jmp loader_jmp

; read(char* buffer, size_t track_num, size_t start_sector, size_t sector_count)
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
; using drive number 0 and side 0
mov dx, 0

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

[section .data]
loader_long_jump:
; long jump op code
db 0eah
; 2nd stage loader init code location
dw 8200h
; system code segment selector
dw 08h

gdt_info:
dw 17h
dd 8000h
