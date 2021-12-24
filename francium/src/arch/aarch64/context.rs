extern "C" {
	fn restore_process_context(ctx: &ProcessContext);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ProcessContext {
	// regs includes kernel sp
	pub regs: [usize; 32],
}

#[repr(C)]
pub struct ExceptionContext {
	// regs includes usermode sp

	pub regs: [usize; 32],
	pub saved_pc: usize,
	pub saved_spsr: usize,
	pub saved_tpidr: usize
}

impl ExceptionContext {
	pub const fn new() -> ExceptionContext {
		ExceptionContext {
			regs: [0; 32],
			saved_pc: 0,
			saved_spsr: 0,
			saved_tpidr: 0
		}
	}
}

impl ProcessContext {
	pub const fn new() -> ProcessContext {
		ProcessContext {
			regs: [0; 32],
		}
	}

	pub unsafe fn switch(&self) {
		restore_process_context(self);
	}
}
