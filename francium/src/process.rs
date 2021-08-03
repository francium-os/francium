use crate::memory::AddressSpace;
use crate::mmu::*;
use crate::arch::aarch64::context::ProcessContext;

pub struct Process {
	pub address_space: AddressSpace,
	pub context: ProcessContext
}

impl Process {
	pub fn new(root: &PageTable) -> Process {
		let p = Process {
			address_space: AddressSpace::new(root.user_process()),
			context: ProcessContext::new()
		};
		p
	}

	pub fn setup_context(&mut self, initial_pc: usize, initial_sp: usize) {
		self.context.regs[31] = initial_sp;
		self.context.saved_pc = initial_pc;
	}

	pub fn switch_to(self) {
		// TODO: arm-ism

		unsafe {
			// Switch to `pages`, switch to user mode
			set_ttbr0_el1(virt_to_phys(&self.address_space.page_table as *const PageTable as usize));
			set_ttbr1_el1(virt_to_phys(&self.address_space.page_table  as *const PageTable as usize));
			self.context.switch();
		}
	}
}