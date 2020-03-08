use core::sync::atomic;
use core::ops;
use core::cell;

use crate::task;

pub struct Mutex<T> {
	guarded: cell::UnsafeCell<T>,
	lock: atomic::AtomicBool
}

pub struct MutexGuard<'a, T> {
	mutex: &'a Mutex<T>
}

impl<T> Mutex<T> {
	pub const fn new(obj: T) -> Mutex<T> {
		Mutex {
			guarded: cell::UnsafeCell::new(obj),
			lock: atomic::AtomicBool::new(false)
		}
	}

	pub fn lock(&self) -> MutexGuard<T> {
		while self.lock.swap(true, atomic::Ordering::SeqCst) {
			task::suspend();
		}

		MutexGuard {
			mutex: self
		}
	}

	pub fn unlock(&self) {
		self.lock.store(false, atomic::Ordering::SeqCst);
	}
}

impl<T> ops::Deref for MutexGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe {
			&*self.mutex.guarded.get()
		}
	}
}

impl<T> ops::DerefMut for MutexGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *self.mutex.guarded.get()
		}
	}
}

impl<T> Drop for MutexGuard<'_, T> {
	fn drop(&mut self) {
		self.mutex.unlock()
	}
}

unsafe impl<T> Send for Mutex<T> {}

unsafe impl<T> Sync for Mutex<T> {}
