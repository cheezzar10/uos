#define NULL 0x0

typedef __SIZE_TYPE__ size_t;

typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned short uint16_t;

void set_cr3(uint32_t* pd_base);

// loader debug related stuff
#define SCRN_BUF_ORG 0xb8000

#define SCRN_COLS 80
#define SCRN_ROWS 24

const char* const SCRN_BUF_END = (char*)(SCRN_BUF_ORG + SCRN_COLS*SCRN_ROWS*2);

const char HEX_DIGITS[16] = "0123456789abcdef";

static size_t row = 0;

static size_t col = 0;

const uint32_t PG_TBL_ENTRY_PRESENT_BIT = 0x1;
const uint32_t PG_TBL_ENTRY_RW_BIT = 0x2;
const uint32_t PG_TBL_ENTRY_MAPPED_BIT = 0x100;

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

void int2hex(uint32_t n, char* buf) {
	for (size_t i = 8;i > 0; i--) {
		buf[i-1] = HEX_DIGITS[n & 0xf];
		n >>= 4;
	}
}
//  the end of loader debug related stuff

// ELF related stuff

// ELF header structure
#define EI_NIDENT 16
struct ElfHeader {
	unsigned char e_ident[EI_NIDENT];
	uint16_t e_type;
	uint16_t e_machine;
	uint32_t e_version;
	// originally, the following fields are addresses, but storing them like that for the moment

	// entry point virtual address
	size_t e_entry;
	// program headers offset
	size_t e_phoff;
	// section headers offset
	size_t e_shoff;

	uint32_t e_flags;
	uint16_t e_hsize;
	uint16_t e_phentsize;
	uint16_t e_phnum;
	uint16_t e_shentsize;
	uint16_t e_shnum;
	uint16_t e_shstrndx;
};

// ELF section header structure
struct ElfSecHeader {
	uint32_t sh_name;
	uint32_t sh_type;
	uint32_t sh_flags;
	
	// the most important 2 fields
	// they are addresses actually, but expressed as machine words
	
	// section virtual address
	size_t sh_addr;
	// section offset inside ELF binary
	size_t sh_offset;

	uint32_t sh_size;
	uint32_t sh_link;
	uint32_t sh_info;
	uint32_t sh_addralign;
	uint32_t sh_entsize;
};

static void init_sys_vm_map(const struct ElfSecHeader sec_hdr[], size_t sec_hdr_num, uint32_t* pg_tbl, uint32_t sys_bin_pg_offset);

void* init_vm(struct ElfHeader* elf_hdr) {
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

	// filling all page table entries with identity mapping by default
	for (size_t i = 0;i < 1024;i++) {
		uint32_t pt_entry = i << 12;

		// configuring present and R/W bits
		pt[i] = pt_entry | 0x3;
	}

	init_sys_vm_map((void*)elf_hdr + elf_hdr->e_shoff, elf_hdr->e_shnum, pt, (uint32_t)elf_hdr >> 12);

	set_cr3(pd_base);

	return (void*)elf_hdr->e_entry;
}

static void init_sys_vm_map(const struct ElfSecHeader sec_hdr[], size_t sec_hdr_num, uint32_t* pg_tbl, uint32_t sys_bin_pg_offset) {
	for (size_t i = 0;i < sec_hdr_num;i++) {
		if (sec_hdr[i].sh_addr == NULL) {
			// skipping null sections
			continue;
		}

		// calculating memory page index for section address
		uint32_t sec_addr_mem_pg = sec_hdr[i].sh_addr >> 12;

		// we'll use bit number 9 of page table entry to mark it as processed
		uint32_t sec_offset_mem_pg = sec_hdr[i].sh_offset >> 12;

		// calculating offset between section offset page in system binary and it's virtual addr page
		int32_t sec_vm_pg_offset = sec_offset_mem_pg - sec_addr_mem_pg;

		// calculating current section last occupied memory page index
		uint32_t sec_end_mem_pg = (sec_hdr[i].sh_addr + sec_hdr[i].sh_size) >> 12;

		for (size_t pg_idx = sec_addr_mem_pg;pg_idx <= sec_end_mem_pg;pg_idx++) {
			uint32_t pg_tbl_entry = pg_tbl[pg_idx];

			// skipping already processed pages
			if (pg_tbl_entry & PG_TBL_ENTRY_MAPPED_BIT) {
				continue;
			}

			pg_tbl_entry = (sys_bin_pg_offset + sec_vm_pg_offset + pg_idx) << 12;
			pg_tbl[pg_idx] = pg_tbl_entry | (PG_TBL_ENTRY_PRESENT_BIT | PG_TBL_ENTRY_RW_BIT | PG_TBL_ENTRY_MAPPED_BIT);
		}
	}
}
