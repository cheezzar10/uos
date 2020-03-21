use core::sync::atomic::{ AtomicUsize, Ordering };
use core::cell::{ RefCell };

const RING_BUF_SIZE: usize = 16;

pub struct RingBuf {
	buf: RefCell<[u8; RING_BUF_SIZE]>,
	rpos: AtomicUsize,
	wpos: AtomicUsize
}

impl RingBuf {
	pub const fn new() -> RingBuf {
		RingBuf { 
			buf: RefCell::new([0; RING_BUF_SIZE]),
			rpos: AtomicUsize::new(0),
			wpos: AtomicUsize::new(0)
		}
	}

	pub fn push_back(&self, b: u8) {
		self.write_buf(b);
		Self::inc(&self.wpos);
	}

	// reducing mutable reference scope
	fn write_buf(&self, b: u8) {
		let mut buf_ref = self.buf.borrow_mut();
		buf_ref[self.wpos.load(Ordering::SeqCst)] = b;
	}

	pub fn pop_front(&self) -> Option<u8> {
		let rpos = self.rpos.load(Ordering::SeqCst);

		if rpos == self.wpos.load(Ordering::SeqCst) {
			None
		} else {
			let rv = Some(self.read_buf(rpos));
			Self::inc(&self.rpos);
			rv
		}
	}

	fn read_buf(&self, pos: usize) -> u8 {
		loop {
			let borrow_attempt_res = self.buf.try_borrow();

			if let Ok(buf_ref) = borrow_attempt_res {
				return buf_ref[pos]
			}
			// TODO here we should spin or suspend current task to make writer chance to make their work
		}
	}

	fn inc(pos: &AtomicUsize) {
		loop {
			let curr_pos = pos.load(Ordering::SeqCst);

			let new_pos = if ((curr_pos + 1) % RING_BUF_SIZE) == 0 {
				// wrapping buffer position around
				0
			} else {
				curr_pos + 1
			};

			let prev_pos = pos.compare_and_swap(curr_pos, new_pos, Ordering::SeqCst);
			if prev_pos == curr_pos {
				break;
			}
		}
	}
}

// may fail actually if writer and reader threads will be overlapped
unsafe impl Sync for RingBuf {}
