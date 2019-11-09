typedef __SIZE_TYPE__ size_t;

typedef unsigned int uint32_t;

void set_cr3(uint32_t* pd_base);

void init_vm() {
	// VM page directory base address starts at 4kb
	uint32_t* pd_base = (uint32_t*)(0x1 << 12);

	// page directory entry for the first 4mb
	// it will store it's page table starting at 8kb
	uint32_t pd_entry = 0x1 << 13;
	// storing page table addr for the first 4mb
	uint32_t* pt = (uint32_t*)pd_entry;
	// configuring present and R/W bits
	pd_entry |= 0x3;

	pd_base[0] = pd_entry;

	// configuring identity mapping for the first 4mb
	// page table located at 8kb

	// system binary loaded at this physical page address
	uint32_t sys_phys_page = 0x9;
	for (size_t i = 0;i < 1024;i++) {
		uint32_t pt_entry;

		switch (i) {
		case 0x0:
		case 0x1:
		case 0x2:
		case 0x3:
		case 0x4:
		case 0x5:
			// remapping system binary pages to it' location in RAM
			pt_entry = (sys_phys_page+i) << 12;
			break;
		default:
			// identity mapping by default
			pt_entry = i << 12;
		}

		// configuring present and R/W bits
		pt[i] = pt_entry | 0x3;
	}

	set_cr3(pd_base);
}
