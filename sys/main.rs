#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::{Write, Result};

#[link(name = "uos")]
extern {
	// external llinkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCREEN_BUF: *mut [u8; 3840];

	fn get_sp() -> *const i32;

	fn register_interrupt_handler(vec_num: usize, handler: extern fn());

	fn register_interrupt_handler_with_err_code(vec_num: usize, handler: extern fn(err_code: usize));

	fn write_byte_to_port(byte: u32, port_num: u32);

	fn read_byte_from_port(port_num: u32) -> u32;

	fn load_idt();

	fn interrupts_enable();
}

static mut SCREEN: ScreenWriter = ScreenWriter { pos: 0 };

const DIVIDE_ERROR_INTR_VEC_NUM: usize = 0;
const GENERAL_PROTECTION_ERR_VEC_NUM: usize = 13;

const TIMER_INTR_VEC_NUM: usize = 32;
const KBD_INTR_VEC_NUM: usize = 33;

const MASTER_ICW1_IOPORT_NUM: u32 = 0x20;
const SLAVE_ICW1_IOPORT_NUM: u32 = 0xa0;

const MASTER_ICW2_IOPORT_NUM: u32 = MASTER_ICW1_IOPORT_NUM + 1;
const SLAVE_ICW2_IOPORT_NUM: u32 = SLAVE_ICW1_IOPORT_NUM + 1;

const CMOS_RAM_CMD_PORT_NUM: u32 = 0x70;
const CMOS_RAM_DATA_PORT_NUM: u32 = 0x71;

const KBD_DATA_IOPORT_NUM: u32 = 0x60;
const KEY_RELEASED_BIT_MASK: u32 = 0x80;
const KEY_SCAN_CODE_MASK: u32 = !KEY_RELEASED_BIT_MASK;

#[no_mangle]
pub unsafe extern fn _start() {
	init();

	loop {}
}

unsafe fn init() {
	SCREEN.clear();

	write!(&mut SCREEN, "stack @{:p}\n", get_sp()).unwrap();

	// registering mandatory interrupt handlers
	register_interrupt_handler(DIVIDE_ERROR_INTR_VEC_NUM, divide_error);
	register_interrupt_handler_with_err_code(GENERAL_PROTECTION_ERR_VEC_NUM, general_protection_error);

	// registering HW interrupt handlers
	register_interrupt_handler(TIMER_INTR_VEC_NUM, timer_intr_handler);
	register_interrupt_handler(KBD_INTR_VEC_NUM, kbd_intr_handler);

	// programmable interrupt controller initialization
	init_pic();

	init_ata_hdd();
}

unsafe fn init_pic() {
	// ICW1 edge triggered mode
	write_byte_to_port(0x11, MASTER_ICW1_IOPORT_NUM);
	write_byte_to_port(0x11, SLAVE_ICW1_IOPORT_NUM);

	// ICW2 assigning interrupt numbers to master and slave controller 
	// ( vectors 32-39 to master and 40-47 to slave )
	write_byte_to_port(32, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(40, SLAVE_ICW2_IOPORT_NUM);

	// ICW3 connecting slave controller to master IRQ2
	write_byte_to_port(0x4, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(0x2, SLAVE_ICW2_IOPORT_NUM);

	// ICW4 x86 processing mode, normal EOI
	write_byte_to_port(0x1, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(0x1, SLAVE_ICW2_IOPORT_NUM );

	// OCW1 unmasking all interrupts
	write_byte_to_port(0, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(0, SLAVE_ICW2_IOPORT_NUM);

	load_idt();

	interrupts_enable();
}

unsafe fn init_ata_hdd() {
	// checking disk type
	write_byte_to_port(0x12, CMOS_RAM_CMD_PORT_NUM);

	let hdd_info = read_byte_from_port(CMOS_RAM_DATA_PORT_NUM);
	if hdd_info & 0xf0 == 0xf0 {
		// getting information about first hard disk from specific register
		write_byte_to_port(0x19, CMOS_RAM_CMD_PORT_NUM);
		let hda_info = read_byte_from_port(CMOS_RAM_DATA_PORT_NUM);

		writeln!(&mut SCREEN, "hda info: {:x}", hda_info).unwrap();
	}
}

unsafe fn end_of_intr_handling() {
	write_byte_to_port(0x20, MASTER_ICW1_IOPORT_NUM);
	write_byte_to_port(0x20, SLAVE_ICW1_IOPORT_NUM);
}

extern fn divide_error() {
	unsafe {
		write!(&mut SCREEN, "divide error").unwrap();
	}
}

extern fn general_protection_error(err_code: usize) {
	unsafe {
		write!(&mut SCREEN, "general protection error: {:x}\n", err_code).unwrap();

		loop {}
	}
}

extern fn timer_intr_handler() {
	unsafe {
		end_of_intr_handling();
	}
}

extern fn kbd_intr_handler() {
	unsafe {
		let key_scan_code = read_byte_from_port(KBD_DATA_IOPORT_NUM);

		// deciding was it key press or key release
		if (key_scan_code & KEY_RELEASED_BIT_MASK) == 0 {
			// bit 7 is clear - key pressed
			write!(&mut SCREEN, "key pressed: {:x}\n", key_scan_code).unwrap();
		} else {
			// bit 7 is set - key released
			write!(&mut SCREEN, "key released: {:x}\n", key_scan_code & KEY_SCAN_CODE_MASK).unwrap();
		}

		end_of_intr_handling();
	}
}

// TODO make this object thread safe
struct ScreenWriter {
	pos: usize
}

impl ScreenWriter {
	unsafe fn print(&mut self, s: &str) {
		for b in s.bytes() {
			if b == b'\n' {
				let next_line_offset = 80 - (self.pos % 80);
				self.pos += next_line_offset;
			} else {
				(*SCREEN_BUF)[self.pos*2] = b;
				self.pos += 1;
			}
		}
	}

	unsafe fn clear(&mut self) {
		for (i, b) in (*SCREEN_BUF).iter_mut().enumerate() {
			*b = if (i & 0x1) == 1 {
				0x7
			} else {
				0x20
			}
		}
		self.pos = 0;
	}
}

impl Write for ScreenWriter {
	fn write_str(&mut self, s: &str) -> Result {
		unsafe {
			self.print(s);
		}
		Ok(())
	}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
	// making some sign that we reached this place
	loop {}
}

// the following functions make linker happy
#[no_mangle]
extern fn rust_eh_personality() {}

#[no_mangle]
extern fn _Unwind_Resume() {}
