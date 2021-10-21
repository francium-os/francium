use alloc::boxed::Box;
use alloc::sync::Arc;
use smallvec::SmallVec;
use spin::Mutex;

use crate::Process;
use crate::process::ProcessState;
use crate::aarch64::context::ExceptionContext;

pub struct Scheduler {
	pub processes: SmallVec<[Arc<Mutex<Box<Process>>>; 4]>,
	pub runnable_processes: SmallVec<[Arc<Mutex<Box<Process>>>; 4]>,
	pub current_process_index: usize
}

lazy_static! {
	static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

impl Scheduler {
	fn new() -> Scheduler {
		Scheduler {
			processes: SmallVec::new(),
			runnable_processes: SmallVec::new(),
			current_process_index: 0
		}
	}

	fn get_current_process(&mut self) -> Arc<Mutex<Box<Process>>> {
		self.runnable_processes[self.current_process_index].clone()
	}

	pub fn get_next_process(&mut self) -> Arc<Mutex<Box<Process>>> {
		if self.current_process_index == self.runnable_processes.len() - 1 {
			self.current_process_index = 0;
		} else {
			self.current_process_index += 1;
		}
		self.runnable_processes[self.current_process_index].clone()
	}

	pub fn tick(&mut self, exc_context: &mut ExceptionContext) {
		if self.runnable_processes.len() == 0 {
			panic!("No runnable processes!");
		}

		// do the thing
		let p = self.runnable_processes[self.current_process_index].clone();

		p.lock().switch_in(exc_context);
		let next = self.get_next_process();
		next.lock().switch_out(exc_context);
	}

	pub fn suspend(&mut self, p: &Arc<Mutex<Box<Process>>>) {
		let process_id = p.lock().id;
		if let Some(runnable_index) = self.runnable_processes.iter().position(|x| x.lock().id == process_id) {
			if runnable_index == self.current_process_index {
				panic!("Bad");
			}
			if runnable_index < self.current_process_index {
				self.current_process_index -= 1;
			}
			self.runnable_processes.remove(runnable_index);
		}

		p.lock().state = ProcessState::Suspended;
	}

	pub fn wake(&mut self, p: Arc<Mutex<Box<Process>>>) {
		let process_id = p.lock().id;
		if let Some(_runnable_index) = self.runnable_processes.iter().position(|x| x.lock().id == process_id) {
			// wtf
			panic!("Trying to re-wake a process!");
		} else {
			self.runnable_processes.push(p);
		}
		// TODO: reschedule, as well?
	}

	pub fn terminate_current_process(&mut self, exc: &mut ExceptionContext) {
		let process = self.get_current_process();

		// Important - don't suspend while we're running! This is probably possible to do properly, but...
		self.tick(exc);
		self.suspend(&process);

		let process_id = process.lock().id;

		let proc_index = self.processes.iter().position(|x| x.lock().id == process_id).unwrap();
		self.processes.remove(proc_index);
	}
}

pub fn tick(exc: &mut ExceptionContext) {
	let mut sched = SCHEDULER.lock();
	sched.tick(exc);
}

pub fn register_process(p: Arc<Mutex<Box<Process>>>) {
	let mut sched = SCHEDULER.lock();
	sched.processes.push(p.clone());
	sched.runnable_processes.push(p.clone());
}

pub fn get_current_process() -> Arc<Mutex<Box<Process>>> {
	let sched = SCHEDULER.lock();
	sched.runnable_processes[sched.current_process_index].clone()
}

pub fn suspend_process(p: Arc<Mutex<Box<Process>>>) {
	let mut sched = SCHEDULER.lock();
	sched.suspend(&p);
}

pub fn wake_process(p: Arc<Mutex<Box<Process>>>) {
	let mut sched = SCHEDULER.lock();
	sched.wake(p);
}

pub fn terminate_current_process(exc: &mut ExceptionContext) {
	let mut sched = SCHEDULER.lock();
	sched.terminate_current_process(exc);
}