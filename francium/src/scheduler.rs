use alloc::boxed::Box;
use alloc::sync::Arc;
use smallvec::SmallVec;
use spin::{Mutex, MutexGuard};

use crate::Process;
use crate::process::ProcessState;
use crate::aarch64::context::ProcessContext;

pub struct Scheduler {
	pub processes: SmallVec<[Arc<Mutex<Box<Process>>>; 4]>,
	pub runnable_processes: SmallVec<[Arc<Mutex<Box<Process>>>; 4]>,
	pub current_process_index: usize
}

lazy_static! {
	static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

// rust says these are ffi unsafe
// they're right but shut
extern "C" {
	fn switch_process_asm(from_context: *mut ProcessContext, to_context: *const ProcessContext, from_process: &Arc<Mutex<Box<Process>>>, to_process: &Arc<Mutex<Box<Process>>>);
}

#[no_mangle]
pub extern "C" fn force_unlock_mutex(mutex: &Arc<Mutex<Box<Process>>>) {
	unsafe {
		mutex.force_unlock();
	}
}

impl Scheduler {
	fn new() -> Scheduler {
		Scheduler {
			processes: SmallVec::new(),
			runnable_processes: SmallVec::new(),
			current_process_index: 0,
		}
	}

	fn get_current_process(&self) -> Arc<Mutex<Box<Process>>> {
		self.runnable_processes[self.current_process_index].clone()
	}

	fn switch_process(&mut self, from: &Arc<Mutex<Box<Process>>>, to: &Arc<Mutex<Box<Process>>>) {
		println!("Switching processes!!");

		// TODO: wow, this sucks
		{
			unsafe {
				// TODO: lol
				SCHEDULER.force_unlock();
			}

			let from_locked = MutexGuard::leak(from.lock());
			let to_locked = MutexGuard::leak(to.lock());

			to_locked.use_pages();

			unsafe {
				switch_process_asm(&mut from_locked.context, &to_locked.context, &from, &to);
			}
		}
	}

	pub fn get_next_process(&mut self) -> Arc<Mutex<Box<Process>>> {
		if self.current_process_index == self.runnable_processes.len() - 1 {
			self.current_process_index = 0;
		} else {
			self.current_process_index += 1;
		}
		self.runnable_processes[self.current_process_index].clone()
	}

	pub fn tick(&mut self) {
		// todo: process things
		if self.runnable_processes.len() == 0 {
			return
		}

		// do the thing
		let this_process = self.get_current_process();
		let next = self.get_next_process();
		self.switch_process(&this_process, &next);
	}

	pub fn suspend(&mut self, p: &Arc<Mutex<Box<Process>>>) {
		p.lock().state = ProcessState::Suspended;

		let process_id = p.lock().id;
		if let Some(runnable_index) = self.runnable_processes.iter().position(|x| x.lock().id == process_id) {
			if runnable_index < self.current_process_index {
				self.current_process_index -= 1;
			}
			self.runnable_processes.remove(runnable_index);

			if runnable_index == self.current_process_index {
				if self.runnable_processes.len() == 0 {
					panic!("Trying to suspend everything.");
				} else {
					let next = self.get_next_process();
					self.switch_process(p, &next);
				}
			}
		}
	}

	pub fn wake(&mut self, p: Arc<Mutex<Box<Process>>>) {
		let process_id = p.lock().id;
		if let Some(_runnable_index) = self.runnable_processes.iter().position(|x| x.lock().id == process_id) {
			// wtf
			panic!("Trying to re-wake a process!");
		} else {
			self.runnable_processes.push(p);
		}
	}

	pub fn terminate_current_process(&mut self) {
		let this_process = self.get_current_process();

		let process_id = this_process.lock().id;
		let proc_index = self.processes.iter().position(|x| x.lock().id == process_id).unwrap();
		self.processes.remove(proc_index);

		self.suspend(&this_process);
	}
}

pub fn tick() {
	let mut sched = SCHEDULER.lock();
	sched.tick();
}

pub fn register_process(p: Arc<Mutex<Box<Process>>>) {
	let mut sched = SCHEDULER.lock();
	sched.processes.push(p.clone());
	sched.runnable_processes.push(p.clone());
}

pub fn get_current_process() -> Arc<Mutex<Box<Process>>> {
	let sched = SCHEDULER.lock();
	sched.get_current_process()
}

pub fn suspend_process(p: Arc<Mutex<Box<Process>>>) {
	let mut sched = SCHEDULER.lock();
	sched.suspend(&p);
}

pub fn suspend_current_process() {
	let mut sched = SCHEDULER.lock();
	let curr = sched.get_current_process();

	sched.suspend(&curr);
}

pub fn wake_process(p: Arc<Mutex<Box<Process>>>) {
	let mut sched = SCHEDULER.lock();
	sched.wake(p);
}

pub fn terminate_current_process() {
	let mut sched = SCHEDULER.lock();
	sched.terminate_current_process();
}
