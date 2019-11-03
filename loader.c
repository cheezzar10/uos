typedef __SIZE_TYPE__ size_t;

typedef unsigned int uint32_t;

// the origin of screen buffer in memory
#define SCRN_BUF_ORG 0xb8000

#define SCRN_COLS 80
#define SCRN_ROWS 24

static size_t row = 0;
static size_t col = 0;

void print(const char* str);

void set_cr3(uint32_t* pd_base);

void init_vm() {
	print("ROBCO UOS boot\n");

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
		if (i == 2) {
			// identity mapping for all pages except system
			// data section remapped from 8kb to 40kb
			pt_entry = 0xa << 12;
		} else if (i == 0xa) {
			// remapping 40k to 36k
			pt_entry = (i-1) << 12;
		}
		// configuring present and R/W bits
		pt_entry |= 0x3;

		pt[i] = pt_entry;
	}

	set_cr3(pd_base);
}

void print(const char* s) {
	char* scrn_buf = (char*)(SCRN_BUF_ORG + 2*80*row + col*2);

	for (size_t si = 0;s[si] != '\0';si++) {
		if (s[si] == '\n') {
			scrn_buf += (SCRN_COLS - col)*2;

			row++;
			col = 0;
		} else {
			*scrn_buf = s[si];
			scrn_buf += 2;
			col++;
		}
	}
}
