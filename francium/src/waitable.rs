use smallvec::SmallVec;
use spin::Mutex;
use alloc::sync::Arc;
use alloc::boxed::Box;
use crate::Thread;
use crate::scheduler;
use crate::handle::Handle;
use core::sync::atomic::{AtomicBool,Ordering};

#[derive(Debug)]
pub struct Waiter {
	waiters: Mutex<SmallVec<[Arc<Thread>; 1]>>,
	pending: AtomicBool,
}

impl Waiter {
	pub fn new() -> Waiter {
		Waiter {
			waiters: Mutex::new(SmallVec::new()),
			pending: AtomicBool::new(false)
		}
	}

	pub fn post_wait(&self) -> bool {
		let pending = self.pending.load(Ordering::Acquire);
		if pending {
			self.pending.store(false, Ordering::Release);
			return true;
		} else {
			self.waiters.lock().push(scheduler::get_current_thread());
			return false;
		}
	}

	pub fn wait(&self) {
		if !self.pending.load(Ordering::Acquire) {
			self.waiters.lock().push(scheduler::get_current_thread());
			scheduler::suspend_current_thread();
		} else {
			self.pending.store(false, Ordering::Release);
		}
	}

	pub fn remove_wait(&self) {
		let mut waiters_locked = self.waiters.lock();

		let pos = waiters_locked.iter().position(|x| x.id == scheduler::get_current_thread().id);
		if let Some(x) = pos {
			waiters_locked.remove(x);
		}
	}

	pub fn signal_one(&self) {
		match self.waiters.lock().pop() {
			Some(waiter) => scheduler::wake_thread(waiter),
			None => self.pending.store(true, Ordering::Release)
		}
	}

	pub fn signal_all(&self) {
		self.waiters.lock().drain(..).map(|x| scheduler::wake_thread(x)).collect()
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

	fn post_wait(&self) -> bool {
		self.get_waiter().post_wait()
	}

	fn remove_wait(&self) {
		self.get_waiter().remove_wait();
	}
}

const MAX_HANDLES: usize = 128;
const INVALID_HANDLE: Handle = Handle::Invalid;

pub fn wait_handles(handles: &[u32]) {
	let mut handle_objects = [INVALID_HANDLE; MAX_HANDLES];
	let mut handle_objects = &mut handle_objects[0..handles.len()];

	{
		let process_locked = scheduler::get_current_process();
		let process = process_locked.lock();

		for (i, handle) in handles.iter().enumerate() {
			handle_objects[i] = process.handle_table.get_object(*handle);
		}
	}

	let mut any_pending = false;

	for handle in handle_objects.iter() {
		match handle {
			// What handles are waitable?
			Handle::Port(port) => {
				any_pending = any_pending || port.post_wait();
			},

			Handle::ServerSession(server_session) => {
				any_pending = any_pending || server_session.post_wait();
			},

			Handle::ClientSession(client_session) => {
				any_pending = any_pending || client_session.post_wait();
			},

			_ => {}
		}
	}

	if !any_pending {
		scheduler::suspend_current_thread();
	}

	for handle in handle_objects.iter() {
		match handle {
			// What handles are waitable?
			Handle::Port(port) => {
				port.remove_wait();
			},

			Handle::ServerSession(server_session) => {
				server_session.remove_wait();
			},

			Handle::ClientSession(client_session) => {
				client_session.remove_wait();
			},

			_ => {}
		}
	}
}