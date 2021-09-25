use alloc::boxed::Box;
use alloc::sync::Arc;
use smallvec::SmallVec;
use spin::Mutex;

use crate::Process;
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

	pub fn tick(&mut self, exc_context: &mut ExceptionContext) {
		if self.runnable_processes.len() == 0 {
			panic!("No runnable processes!");
		}

		// do the thing
		let p = self.runnable_processes[self.current_process_index].clone();

		p.lock().switch_in(exc_context);
		let next = self.advance();
		next.lock().switch_out(exc_context);
	}

	pub fn advance(&mut self) -> Arc<Mutex<Box<Process>>> {
		if self.current_process_index == self.runnable_processes.len() - 1 {
			self.current_process_index = 0;
		} else {
			self.current_process_index += 1;
		}
		self.runnable_processes[self.current_process_index].clone()
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