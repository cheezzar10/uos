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
}

static mut SCREEN: ScreenWriter = ScreenWriter { pos: 0 };

#[no_mangle]
pub unsafe extern fn _start(intr_vec_num: i32) {
	if intr_vec_num == -1 {
		init();
	} else {
		// TODO interrupt handling dispatch will be here
	}

	loop {}
}

unsafe fn init() {
	SCREEN.clear();

	if let Ok(()) = write!(&mut SCREEN, "stack @{:p}\n", get_sp()) {}
}

// TODO make this object thread safe
struct ScreenWriter {
	pos: usize
}

impl ScreenWriter {
	unsafe fn print(&mut self, s: &str) {
		for b in s.bytes() {
			(*SCREEN_BUF)[self.pos*2] = b;
			self.pos += 1;
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
