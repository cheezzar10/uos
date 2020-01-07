bootldr_dir = boot
sys_dir = sys

ASFLAGS = --32
CFLAGS = -m32 -Wall -std=c99 -O0 -fno-builtin -nostdlib
LDFLAGS := -T$(bootldr_dir)/loader.ld
ARFLAGS := ru
RANLIB := ranlib
RUSTC := rustc
RUSTCFLAGS = --edition=2018 --emit=link -C panic=abort -C link-arg=-nostartfiles -C debuginfo=0 -L. --crate-name

.PHONY: all
all: uos.img

uos.img: loader.bin uos
	mkdosfs -n UOS -C $@ -S 512 1440
	dd if=mbr.com of=$@ bs=1 seek=62
	dd if=$< of=$@ bs=512 seek=1
	dd if=$(word 2, $^) of=$@ bs=512 seek=18
	dd if=/dev/zero of=$@ bs=512 seek=2879 count=1

.INTERMEDIATE: loader.bin
loader.bin: loader mbr.com
	objcopy -Obinary $< $@

vpath %.rs $(sys_dir)
vpath %.c $(sys_dir)
vpath %.s $(sys_dir)

.INTERMEDIATE: uos
uos: main.rs libuos.rlib
	$(RUSTC) $(RUSTCFLAGS) $@ $<
	strip $@

.INTERMEDIATE: libuos.rlib
libuos.rlib: lib.rs libuos.a
	$(RUSTC) --crate-type lib $(RUSTCFLAGS) $(subst lib,, $(basename $@)) $<

.INTERMEDIATE: libuos.a
libuos.a: arch.o uos.o
	$(AR) $(ARFLAGS) $@ $?
	$(RANLIB) $@

.INTERMEDIATE: uos.o
uos.o: uos.c

.INTERMEDIATE: arch.o
arch.o: arch.s

vpath %.asm $(bootldr_dir)
vpath %.s $(bootldr_dir)
vpath %.c $(bootldr_dir)

.INTERMEDIATE: loader
loader: ldrinit.o loader.o
	$(LD) $(LDFLAGS) -o $@ $^

.INTERMEDIATE: mbr.com
mbr.com: mbr.asm
	nasm -f bin -o $@ $^

.INTERMEDIATE: ldrinit.o
ldrinit.o: ldrinit.s

.INTERMEDIATE: loader.o
loader.o: loader.c

.PHONY: rebuild
rebuild: clean all

.PHONY: clean
clean:
	$(RM) -f *.img *.a *.rlib
