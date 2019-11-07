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
		// identity mapping by default
		uint32_t pt_entry = i << 12;
		if (i <= 3) {
			// remapping system binary pages to address 0x0
			pt_entry = (sys_phys_page+i) << 12;
		} else if (i > 3 && i <= 5) {
			pt_entry = (sys_phys_page+i-1) << 12;
		}
		// configuring present and R/W bits
		pt_entry |= 0x3;

		pt[i] = pt_entry;
	}

	set_cr3(pd_base);
}
