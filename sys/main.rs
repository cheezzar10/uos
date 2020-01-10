#![no_std]
#![no_main]

#[macro_use]
extern crate uos;

use core::panic::PanicInfo;

use uos::console;

#[link(name = "uos")]
extern {
	fn get_sp() -> *const i32;

	fn register_interrupt_handler(vec_num: usize, handler: extern fn());

	fn register_interrupt_handler_with_err_code(vec_num: usize, handler: extern fn(err_code: usize));

	fn write_byte_to_port(byte: u32, port_num: u32);

	fn read_byte_from_port(port_num: u32) -> u32;

	fn load_idt();

	fn interrupts_enable();
}

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
	console::clear();

	console_println!("stack @{:p}", get_sp());

	// registering mandatory interrupt handlers
	register_interrupt_handler(DIVIDE_ERROR_INTR_VEC_NUM, divide_error);
	register_interrupt_handler_with_err_code(GENERAL_PROTECTION_ERR_VEC_NUM, general_protection_error);

	// registering HW interrupt handlers
	register_interrupt_handler(TIMER_INTR_VEC_NUM, timer_intr_handler);
	register_interrupt_handler(KBD_INTR_VEC_NUM, kbd_intr_handler);

	// programmable interrupt controller initialization
	init_pic();

	init_ata_hdd();

	create_thread(idle_thread);

	console_println!("init: yielding");
	thread_yield();

	console_println!("init: resumed");
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

		console_println!("hda info: {:x}", hda_info);
	}
}

unsafe fn end_of_intr_handling() {
	write_byte_to_port(0x20, MASTER_ICW1_IOPORT_NUM);
	write_byte_to_port(0x20, SLAVE_ICW1_IOPORT_NUM);
}

extern fn divide_error() {
	console_println!("divide error");
}

extern fn general_protection_error(err_code: usize) {
	console_println!("general protection error: {:x}", err_code);

	loop {}
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
			console_println!("key pressed: {:x}", key_scan_code);
		} else {
			// bit 7 is set - key released
			console_println!("key released: {:x}", key_scan_code & KEY_SCAN_CODE_MASK);
		}

		end_of_intr_handling();
	}
}

// TODO move to dedicated threading library
fn create_thread(f: fn()) {
	console_println!("creating new thread: {:p}", f);
}

fn thread_yield() {
}

fn idle_thread() {
	console_println!("idle: running");

	for _ in 0..500000 {
	}

	console_println!("idle: yielding");

	thread_yield();

	console_println!("idle: exiting");
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
