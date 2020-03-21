use core::fmt;

use crate::task;
use crate::ring;

#[link(name = "uos")]
extern {
	// external linkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCR_BUF: *mut [u8; 3840];
}

static mut SCR_WRITER: ScreenWriter = ScreenWriter { pos: 0 };

pub static KBD_BUF: ring::RingBuf = ring::RingBuf::new();

// TODO make this object thread safe
pub struct ScreenWriter {
	pos: usize
}

impl ScreenWriter {
	unsafe fn print(&mut self, s: &str) {
		for chr in s.bytes() {
			self.write_char(chr);
		}
	}

	unsafe fn write_char(&mut self, chr: u8) {
		if chr == b'\n' {
			let next_line_offset = 80 - (self.pos % 80);
			self.pos += next_line_offset;
		} else {
			(*SCR_BUF)[self.pos*2] = chr;
			self.pos += 1;
		}
	}

	unsafe fn clear(&mut self) {
		for (i, b) in (*SCR_BUF).iter_mut().enumerate() {
			*b = if (i & 0x1) == 1 {
				0x7
			} else {
				0x20
			}
		}
		self.pos = 0;
	}
}

impl fmt::Write for ScreenWriter {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		unsafe {
			self.print(s);
		}
		Ok(())
	}
}

pub unsafe fn clear() {
	SCR_WRITER.clear();
}

pub fn print(args: fmt::Arguments) {
	unsafe {
		fmt::write(&mut SCR_WRITER, args).unwrap();
	}
}

pub fn print_str(s: &str) {
	unsafe {
		SCR_WRITER.print(s);
	}
}

pub fn read_char() -> u8 {
	loop {
		let chr = match KBD_BUF.pop_front() {
			Some(c) => c,
			_ => {
				task::suspend();
				continue
			}
		};

		unsafe {
			// echoing pressed character on the console
			SCR_WRITER.write_char(chr);
		}

		return chr
	}
}
