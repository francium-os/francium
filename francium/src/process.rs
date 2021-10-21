use crate::memory::AddressSpace;
use crate::arch::aarch64::context::ProcessContext;
use crate::aarch64::context::ExceptionContext;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use crate::handle_table::HandleTable;

#[derive(Debug)]
pub enum ProcessState {
	Created,
	Runnable,
	Suspended
}

#[derive(Debug)]
pub struct Process {
	pub address_space: Box<AddressSpace>,
	pub context: ProcessContext,
	pub state: ProcessState,
	pub id: usize,
	pub handle_table: HandleTable
}

extern "C" {
	pub fn get_elr_el1() -> usize;
	fn set_elr_el1(val: usize);
	fn get_spsr_el1() -> usize;
	fn set_spsr_el1(val: usize);
	fn get_sp_el0() -> usize;
	fn set_sp_el0(val: usize);
}

static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);

impl Process {
	pub fn new(aspace: Box<AddressSpace>) -> Process {
		let p = Process {
			address_space: aspace,
			context: ProcessContext::new(),
			state: ProcessState::Created,
			id: PROCESS_ID.fetch_add(1, Ordering::SeqCst),
			handle_table: HandleTable::new()
		};

		p
	}

	pub fn setup_context(&mut self, initial_pc: usize, initial_sp: usize) {
		self.context.regs[31] = initial_sp;
		self.context.saved_pc = initial_pc;
	}

	pub fn switch_out(&mut self, exc: &mut ExceptionContext) {
		exc.regs = self.context.regs;
		unsafe {
			set_elr_el1(self.context.saved_pc);
			set_spsr_el1(self.context.saved_spsr);
			set_sp_el0(self.context.regs[31]);
		}

		self.address_space.make_active();
	}

	pub fn switch_in(&mut self, exc: &mut ExceptionContext) {
		self.context.regs = exc.regs;

		unsafe {
			self.context.saved_pc = get_elr_el1();
			self.context.saved_spsr = get_spsr_el1();
			self.context.regs[31] = get_sp_el0();
		}
	}

	pub fn use_pages(&self) {
		self.address_space.make_active();
	}
}

pub fn force_switch_to(locked: Arc<Mutex<Box<Process>>>) {
	let process_context = { 
		let p = locked.lock();
		p.address_space.make_active();
		p.context.clone()
	};
	process_context.switch();
}