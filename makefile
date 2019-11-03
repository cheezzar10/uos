ASFLAGS = --32
CFLAGS = -m32 -Wall -std=c99 -O0 -fno-builtin -nostdlib
LDFLAGS := -Tloader.ld
ARFLAGS := ru
RANLIB := ranlib
RUSTC := rustc
RUSTCFLAGS = --edition=2018 --emit=link -C panic=abort -C link-arg=-nostartfiles -L. --crate-name

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

.INTERMEDIATE: uos
uos: main.rs libuos.a
	$(RUSTC) $(RUSTCFLAGS) $@ $<

.INTERMEDIATE: libuos.a
libuos.a: uos.o
	$(AR) $(ARFLAGS) $@ $?
	$(RANLIB) $@

.INTERMEDIATE: uos.o
uos.o: uos.s

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

.PHONY: clean
clean:
	rm -f *.img *.a uos
