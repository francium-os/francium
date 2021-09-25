use crate::memory::AddressSpace;
use crate::arch::aarch64::context::ProcessContext;
use crate::aarch64::context::ExceptionContext;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;

pub enum ProcessState {
	Created,
	Runnable
}

pub struct Process {
	pub address_space: Box<AddressSpace>,
	pub context: ProcessContext,
	pub state: ProcessState
}

extern "C" {
	fn get_spsr_el1() -> usize;
	fn get_sp_el0() -> usize;
}

impl Process {
	pub fn new(aspace: Box<AddressSpace>) -> Process {
		let p = Process {
			address_space: aspace,
			context: ProcessContext::new(),
			state: ProcessState::Created
		};

		p
	}

	pub fn setup_context(&mut self, initial_pc: usize, initial_sp: usize) {
		self.context.regs[31] = initial_sp;
		self.context.saved_pc = initial_pc;
	}

	pub fn switch_out(&mut self) {
		self.address_space.make_active();
		self.context.switch();
	}

	pub fn switch_in(&mut self, exc: &mut ExceptionContext) {
		self.context.regs = exc.regs;
		self.context.saved_pc = exc.saved_pc;
		unsafe {
			self.context.saved_spsr = get_spsr_el1();
			self.context.regs[31] = get_sp_el0();
		}
	}

	pub fn use_pages(&self) {
		self.address_space.make_active();
	}
}

pub fn switch_locked(locked: Arc<Mutex<Box<Process>>>) {
	let process_context = { 
		let p = locked.lock();
		p.address_space.make_active();
		p.context.clone()
	};
	process_context.switch();
}