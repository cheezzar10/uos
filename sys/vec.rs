use core::mem;
use core::ptr;
use core::ops;
use core::slice;

use crate::alloc;

pub struct Vec<T> {
	buf: *mut T,
	len: usize,
	cap: usize,
}

impl<T> Vec<T> {
	pub const fn new() -> Vec<T> {
		Vec {
			buf: ptr::null_mut(),
			len: 0,
			cap: 0
		}
	}

	pub fn with_cap(cap: usize) -> Vec<T> {
		let alloc_bytes = cap.checked_mul(mem::size_of::<T>());

		match alloc_bytes {
			Some(cap_bytes) => {
				Vec { 
					buf: alloc::alloc(cap_bytes) as *mut T, 
					len: 0, 
					cap: cap 
				}
			},
			_ => panic!("vector capacity overflow!")
		}
	}

	pub fn reserve(&mut self, add: usize) {
		let new_cap = self.len + add;

		let alloc_bytes = new_cap.checked_mul(mem::size_of::<T>());
		match alloc_bytes {
			Some(new_cap_bytes) => {
				let new_buf: *mut T = alloc::alloc(new_cap_bytes) as *mut T;

				if !self.buf.is_null() {
					unsafe {
						ptr::copy_nonoverlapping(self.buf, new_buf, self.len);
					}

					alloc::dealloc(self.buf as *mut u8);
				}

				self.buf = new_buf;
				self.cap = new_cap;
			},
			_ => panic!("vector capacity overflow")
		}
	}

	pub fn push(&mut self, val: T) {
		if self.len == self.cap {
			if self.cap == 0 {
				self.reserve(2);
			} else {
				self.reserve(self.cap);
			}
		}

		let val_addr = self.buf.wrapping_add(self.len);

		unsafe {
			ptr::write(val_addr, val);
		}

		self.len += 1;
	}

	pub fn swap(&mut self, i: usize, j: usize) {
		if i >= self.len || j >= self.len {
			return ()
		}

		if i != j {
			let i_addr = self.buf.wrapping_add(i);
			let j_addr = self.buf.wrapping_add(j);

			unsafe {
				ptr::swap_nonoverlapping(i_addr, j_addr, 1);
			}
		}
	}

	pub fn swap_remove(&mut self, i: usize) -> Option<T> {
		if i >= self.len {
			return None
		}

		self.swap(i, self.len - 1);

		self.pop()
	}

	pub fn pop(&mut self) -> Option<T> {
		if self.len == 0 {
			None
		} else {
			// we can use mem::uninitialized or zeroed to create temporary value
			self.len -= 1;
			let val_ptr = self.buf.wrapping_add(self.len);

			unsafe {
				// may be we should use mem::forget/mem::drop to properly release
				Some(ptr::read(val_ptr))
			}
		}
	}

	pub fn clear(&mut self) {
		self.len = 0;
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn cap(&self) -> usize {
		self.cap
	}
}

impl<T> ops::Deref for Vec<T> {
	type Target = [T];

	fn deref(&self) -> &[T] {
		unsafe {
			slice::from_raw_parts(self.buf, self.len)
		}
	}
}

impl<T> ops::DerefMut for Vec<T> {
	fn deref_mut(&mut self) -> &mut [T] {
		unsafe {
			slice::from_raw_parts_mut(self.buf, self.len)
		}
	}
}

impl<T> Drop for Vec<T> {
	fn drop(&mut self) {
		alloc::dealloc(self.buf as *mut u8);
	}
}
