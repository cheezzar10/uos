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
}

static mut SCREEN: ScreenWriter = ScreenWriter { pos: 0 };

const DIVIDE_ERROR_INTR_VEC_NUM: usize = 0;
const GENERAL_PROTECTION_ERR_VEC_NUM: usize = 13;

#[no_mangle]
pub unsafe extern fn _start() {
	init();

	loop {}
}

unsafe fn init() {
	SCREEN.clear();

	write!(&mut SCREEN, "stack @{:p}\n", get_sp()).unwrap();

	register_interrupt_handler(DIVIDE_ERROR_INTR_VEC_NUM, divide_error);

	register_interrupt_handler_with_err_code(GENERAL_PROTECTION_ERR_VEC_NUM, general_protection_error);
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
