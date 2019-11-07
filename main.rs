#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::{Write, Result};

#[link(name = "uos")]
extern {
	// external llinkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCREEN_BUF: *mut [u8; 3840];

	fn get_sp() -> *const usize;
}

#[no_mangle]
pub extern fn _start() -> ! {
	let mut scrn_buf = ScreenBuf { pos: 0 };

	unsafe {
		(*SCREEN_BUF)[2] = b'$';
		if let Ok(()) = write!(&mut scrn_buf, "stack ptr: {:p}", get_sp()) {}
	}

	loop {}
}

struct ScreenBuf {
	pos: usize
}

impl ScreenBuf {
	unsafe fn print(&mut self, s: &str) {
		for b in s.bytes() {
			(*SCREEN_BUF)[self.pos*2] = b;
			self.pos += 1;
		}
	}
}

impl Write for ScreenBuf {
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
