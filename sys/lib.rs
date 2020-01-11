#![no_std]

#[link(name = "uos")]
extern {
	// external llinkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCR_BUF: *mut [u8; 3840];
}

#[macro_export]
macro_rules! console_println {
	( $f:expr ) => ( { ::uos::console::print(format_args!(concat!($f, "\n"))); } );
	( $f:expr, $( $a:expr ), * ) => ( { ::uos::console::print(format_args!(concat!($f, "\n"), $( $a ), *)); } )
}

pub mod console {
	static mut SCR_WRITER: ScreenWriter = ScreenWriter { pos: 0 };

	// TODO make this object thread safe
	pub struct ScreenWriter {
		pos: usize
	}

	impl ScreenWriter {
		unsafe fn print(&mut self, s: &str) {
			for b in s.bytes() {
				if b == b'\n' {
					let next_line_offset = 80 - (self.pos % 80);
					self.pos += next_line_offset;
				} else {
					(*super::SCR_BUF)[self.pos*2] = b;
					self.pos += 1;
				}
			}
		}

		unsafe fn clear(&mut self) {
			for (i, b) in (*super::SCR_BUF).iter_mut().enumerate() {
				*b = if (i & 0x1) == 1 {
					0x7
				} else {
					0x20
				}
			}
			self.pos = 0;
		}
	}

	impl ::core::fmt::Write for ScreenWriter {
		fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
			unsafe {
				self.print(s);
			}
			Ok(())
		}
	}

	pub unsafe fn clear() {
		SCR_WRITER.clear();
	}

	pub fn print(args: ::core::fmt::Arguments) {
		unsafe {
			::core::fmt::write(&mut SCR_WRITER, args).unwrap();
		}
	}

}
