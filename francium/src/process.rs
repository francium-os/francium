use crate::memory::AddressSpace;
use crate::arch::aarch64::context::ProcessContext;
use alloc::boxed::Box;

pub struct Process {
	pub address_space: Box<AddressSpace>,
	pub context: ProcessContext
}

impl Process {
	pub fn new(aspace: Box<AddressSpace>) -> Process {
		let p = Process {
			address_space: aspace,
			context: ProcessContext::new()
		};

		p
	}

	pub fn setup_context(&mut self, initial_pc: usize, initial_sp: usize) {
		self.context.regs[31] = initial_sp;
		self.context.saved_pc = initial_pc;
	}

	pub fn switch_to(&self) {
		// TODO: arm-ism

		// Switch to `pages`, switch to user mode
		self.address_space.make_active();
		self.context.switch();
	}

	pub fn use_pages(&self) {
		self.address_space.make_active();
	}
}