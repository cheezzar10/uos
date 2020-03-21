#![no_std]
#![no_main]

#[macro_use]
extern crate uos;

use core::panic::PanicInfo;
use core::usize;

use core::sync::atomic::{ AtomicBool, Ordering };

use uos::console;
use uos::task;
use uos::pio;
use uos::intr;

const DIVIDE_ERROR_INTR_VEC_NUM: usize = 0;
const GENERAL_PROTECTION_ERR_VEC_NUM: usize = 13;

const TIMER_INTR_VEC_NUM: usize = 32;
const KBD_INTR_VEC_NUM: usize = 33;

const CMOS_RAM_CMD_PORT_NUM: u32 = 0x70;
const CMOS_RAM_DATA_PORT_NUM: u32 = 0x71;

const KBD_DATA_IOPORT_NUM: u32 = 0x60;
const KEY_RELEASED_BIT_MASK: u32 = 0x80;

// required for modifier key release case handling
// const KEY_SCAN_CODE_MASK: u32 = !KEY_RELEASED_BIT_MASK;

// standard key codes
static KBD_SCAN_CODES: [u8; 83] = [ 0, 0, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 0, 0, b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 0, 0, 0, 
		b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

static SUSPEND_IDLE_TASK: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub unsafe extern fn _start() {
	init();
}

unsafe fn init() {
	console::clear();

	// let kbd_buf_ptr: *const usize = 0xc2b0 as *const usize;
	// let mem = slice::from_raw_parts(kbd_buf_ptr, 8);

	// registering mandatory interrupt handlers
	intr::register_handler(DIVIDE_ERROR_INTR_VEC_NUM, divide_error);
	intr::register_handler_with_err_code(GENERAL_PROTECTION_ERR_VEC_NUM, general_protection_error);

	// registering HW interrupt handlers
	intr::register_handler(TIMER_INTR_VEC_NUM, timer_intr_handler);
	intr::register_handler(KBD_INTR_VEC_NUM, kbd_intr_handler);

	// programmable interrupt controller initialization
	intr::init_pic();

	init_ata_hdd();

	//making this the first kernel task with tid = 0
	task::init_curr_task(0);

	task::create(idle_thread);

	console::print_str("> ");
	loop {
		let _chr = console::read_char();
	}
}

unsafe fn init_ata_hdd() {
	// checking disk type
	pio::out_byte(0x12, CMOS_RAM_CMD_PORT_NUM);

	let hdd_info = pio::in_byte(CMOS_RAM_DATA_PORT_NUM);
	if hdd_info & 0xf0 == 0xf0 {
		// getting information about first hard disk from specific register
		pio::out_byte(0x19, CMOS_RAM_CMD_PORT_NUM);
		let hda_info = pio::in_byte(CMOS_RAM_DATA_PORT_NUM);

		console_println!("hda info: {:x}", hda_info);
	}
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
		intr::eoi();
	}
}

extern fn kbd_intr_handler() {
	unsafe {
		let key_scan_code = pio::in_byte(KBD_DATA_IOPORT_NUM);

		// deciding was it key press or key release
		if (key_scan_code & KEY_RELEASED_BIT_MASK) == 0 {
			// TODO scan table range check
			console::KBD_BUF.push_back(KBD_SCAN_CODES[key_scan_code as usize]);

			// setting suspend idle task flag to give a chance to drain keyboard buffer
			SUSPEND_IDLE_TASK.store(true, Ordering::SeqCst);
		} else {
			// TODO reset modifier key state ( i.e. Shift key released )
		}

		intr::eoi();
	}
}

fn idle_thread() {
	loop {
		if SUSPEND_IDLE_TASK.load(Ordering::SeqCst) {
			// reset idle task suspend flag, making idle task eligible for scheduling again
			SUSPEND_IDLE_TASK.store(false, Ordering::SeqCst);

			task::suspend();
		}
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
