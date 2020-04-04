use core::fmt;

use crate::task;
use crate::ring;
use crate::pio;
use crate::vec;

#[link(name = "uos")]
extern {
	// external linkage for screen buffer memory area making compiler happy 
	// (mutable pointer can't be shared between threads safely)
	static SCR_BUF: *mut [u8; 3840];
}

static mut SCR_WRITER: ScreenWriter = ScreenWriter { pos: 0 };

const SCREEN_COLS: usize = 80;
// re-mappped BIOS data area location
const BIOS_DATA_AREA_ADDR: usize = 0x30400;

const VIDEO_PAGE_0_CURSOR_POS_ADDR: usize = BIOS_DATA_AREA_ADDR + 80;

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
			let next_line_offset = SCREEN_COLS - (self.pos % SCREEN_COLS);
			self.pos += next_line_offset;
		} else {
			(*SCR_BUF)[self.pos*2] = chr;
			self.pos += 1;
		}

		self.move_cursor();
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

	unsafe fn move_cursor(&mut self) {
		// requesting write for cursor position high byte
		pio::out_byte(0xe, 0x3d4);
		pio::out_byte((self.pos >> 8) as u32, 0x3d5);

		// requesting cursor position low byte write
		pio::out_byte(0xf, 0x3d4);
		pio::out_byte((self.pos & 0xff) as u32, 0x3d5);
	}

	unsafe fn get_cursor_pos(&self) -> (usize, usize) {
		let cursor_pos_ptr: *const u16 = VIDEO_PAGE_0_CURSOR_POS_ADDR as *const u16;

		let cursor_col = *cursor_pos_ptr & 0xff;
		let cursor_row = *cursor_pos_ptr >> 8;

		(cursor_row as usize, cursor_col as usize)
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

pub fn get_cursor_pos() -> (usize, usize) {
	unsafe {
		SCR_WRITER.get_cursor_pos()
	}
}

pub fn read_line(buf: &mut vec::Vec<u8>) {
	loop {
		let chr = read_char();
		if chr == b'\n' {
			break;
		}

		buf.push(chr);
	}
}
