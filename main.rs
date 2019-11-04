#![no_std]
#![no_main]

use core::panic::PanicInfo;

// static SCREEN_BUF: *mut [u8; 3840] = 0xb8000 as *mut [u8; 3840];
#[link(name = "uos")]
extern {
	// external llinkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCREEN_BUF: *mut [u8; 3840];
}

#[no_mangle]
pub extern fn _start() -> ! {
	unsafe {
		(*SCREEN_BUF)[0] = b'@';
	}

	loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
	// making some sign that we reached this place
	loop {}
}
