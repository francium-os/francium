use smallvec::SmallVec;
use spin::Mutex;
use alloc::sync::Arc;
use alloc::boxed::Box;
use crate::Process;
use crate::scheduler;
use core::sync::atomic::{AtomicBool,Ordering};

#[derive(Debug)]
pub struct Waiter {
	waiters: Mutex<SmallVec<[Arc<Mutex<Box<Process>>>; 1]>>,
	pending: AtomicBool,
}

impl Waiter {
	pub fn new() -> Waiter {
		Waiter {
			waiters: Mutex::new(SmallVec::new()),
			pending: AtomicBool::new(false)
		}
	}

	pub fn wait(&self) {
		if !self.pending.load(Ordering::Acquire) {
			self.waiters.lock().push(scheduler::get_current_process());
			scheduler::suspend_current_process();
		} else {
			self.pending.store(false, Ordering::Release);
		}
	}

	pub fn signal_one(&self) {
		match self.waiters.lock().pop() {
			Some(waiter) => scheduler::wake_process(waiter),
			None => self.pending.store(true, Ordering::Release)
		}
	}

	pub fn signal_all(&self) {
		self.waiters.lock().drain(..).map(|x| scheduler::wake_process(x)).collect()
	}
}

pub trait Waitable {
	fn get_waiter(&self) -> &Waiter;

	fn wait(&self) {
		self.get_waiter().wait();
	}

	fn signal_one(&self) {
		self.get_waiter().signal_one();
	}

	fn signal_all(&self) {
		self.get_waiter().signal_all();
	}
}