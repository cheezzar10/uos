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

size_t strlen(const char* str) {
	size_t i = 0;
	for (;str[i] != '\0';i++);
	return i;
}

int strcmp(const char* s1, const char* s2) {
	size_t s1_len = strlen(s1);
	size_t s2_len = strlen(s2);

	size_t n = s1_len;
	if (s2_len < s1_len) {
		n = s2_len;
	}

	size_t i = 0;
	for(;i < n && s1[i] == s2[i];i++);

	if (s1[i] == s2[i]) {
		return 0;
	} else if (s1[i] < s2[i]) {
		return -1;
	} else {
		return 1;
	}
}

void* memset(void* dst, int c, size_t n) {
	char* d = dst;
	for (size_t i = 0;i < n;i++) {
		d[i] = c;
	}
	return dst;
}

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

struct BssData {
	size_t addr;
	size_t size;
};

static void init_sys_vm_map(const struct ElfHeader* elf_hdr, uint32_t* pg_tbl, struct BssData* bss_data);

static void print_mem_dump(int32_t* mem, size_t len);

void* init_vm(struct ElfHeader* elf_hdr, struct BssData* bss_data) {
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

	// mapping BIOS data area at the 0x30000
	size_t bios_data_area_pg_idx = 0x30000 >> 12;
	pt[bios_data_area_pg_idx] = 0x3;

	init_sys_vm_map(elf_hdr, pt, bss_data);

	set_cr3(pd_base);

	print_mem_dump((int32_t*)0x1b2b0, 8);

	// TODO drop return value and pass a struct instead which will contain ( entry point address, bss section virtual address & virtual section size )
	return (void*)elf_hdr->e_entry;
}

void print_mem_dump(int32_t* mem, size_t len) {
	print("\n");
	for (size_t i = 0; i < len; i++) {
		char hex_buf[] = "0x00000000";

		int2hex(mem[i], &hex_buf[2]);
		print(hex_buf);
		print(" ");
	}
	print("\n");
}

static void init_sys_vm_map(const struct ElfHeader* elf_hdr, uint32_t* pg_tbl, struct BssData* bss_data) {
	const struct ElfSecHeader* sec_hdr = (void*)elf_hdr + elf_hdr->e_shoff;
	const uint32_t sys_bin_pg_offset = (uint32_t)elf_hdr >> 12;

	for (size_t i = 0;i < elf_hdr->e_shnum;i++) {
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

			print("page ");

			char page_idx_buf[] = "0x00000000";
			int2hex(pg_idx, &page_idx_buf[2]);
			print(page_idx_buf);

			print(" -> ");

			char phys_page_idx_addr[] = "0x00000000";
			int2hex(pg_tbl_entry >> 12, &phys_page_idx_addr[2]);
			print(phys_page_idx_addr);

			print("\n");
		}

		const struct ElfSecHeader* str_tbl_sec = &sec_hdr[elf_hdr->e_shstrndx];
		const char* sec_name = ((void*)elf_hdr + str_tbl_sec->sh_offset) + sec_hdr[i].sh_name;

		if (!strcmp(sec_name, ".bss")) {
			char hex_buf[] = "0x00000000";
			print("bss section addr: ");
			int2hex(sec_hdr[i].sh_addr, &hex_buf[2]);
			print(hex_buf);

			print("\n");

			bss_data->addr = sec_hdr[i].sh_addr;
			bss_data->size = sec_hdr[i].sh_size;
		}
	}
}
