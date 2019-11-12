OUTPUT_FORMAT ("elf32-i386", "elf32-i386", "elf32-i386")
OUTPUT_ARCH(i386)
SECTIONS {
	. = 32K;

	.data : {
		*(.data)
		*(.rodata)
	}

	.bss : {
		*(.bss)
	}

	. = ALIGN(512);

	.text : {
		*(.text)

		. = . + 1K;
		STACK_LIMIT = .;

		STACK_TOP = STACK_LIMIT - 4;
	}
}
