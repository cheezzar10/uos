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
	for (size_t i = 0;i < 1024;i++) {
		uint32_t pt_entry = i << 12;
		if (i == 3) {
			// identity mapping for all pages except system
			// data section remapped from 12kb to 44kb
			pt_entry = 0xb << 12;
		} else if (i == 0xb) {
			// remapping 44k to 40k
			pt_entry = (i-1) << 12;
		}
		// configuring present and R/W bits
		pt_entry |= 0x3;

		pt[i] = pt_entry;
	}

	set_cr3(pd_base);
}
