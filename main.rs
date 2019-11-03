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
		(*SCREEN_BUF)[1] = 0x07;
	}

	loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
	// making some sign that we reached this place
	loop {}
}

/*
unsafe fn get_screen_buf() -> &'static mut [u8; 3840] {
	let screen_buf_ptr = 0xb8000 as *mut [u8; 3840];
	&mut *screen_buf_ptr
}
*/
