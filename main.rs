#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[link(name = "uos")]
extern {
	// external llinkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCREEN_BUF: *mut [u8; 3840];
}

#[no_mangle]
pub extern fn _start() -> ! {
	unsafe {
		print("rust eats metal");
	}

	loop {}
}

unsafe fn print(s: &str) {
	for (i, b) in s.bytes().enumerate() {
		(*SCREEN_BUF)[i*2] = b;
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
