extern "C" {
	fn restore_context(ctx: &ProcessContext);
}

#[repr(C)]
pub struct ProcessContext {
	pub regs: [usize; 32],
	pub saved_pc: usize,
	pub saved_spsr: usize
}

// to enter user mode for the first time: setup SPSR_EL1, setup ELR_EL1, setup SP_EL0, eret

impl ProcessContext {
	pub fn new() -> ProcessContext {
		ProcessContext {
			regs: [0; 32],
			saved_pc: 0,
			saved_spsr: 0
		}
	}

	pub fn switch(self: &ProcessContext) {
		unsafe {
			restore_context(self);
		}
	}
}
