use alloc::sync::Arc;
use crate::process::Thread;

pub struct PerCpuData {
	pub per_cpu_ptr: usize,
	pub saved_kernel_stack: usize,
	pub current_thread: Option<Arc<Thread>>
}

static mut PER_CPU_SINGLE_CORE: PerCpuData = PerCpuData {
	per_cpu_ptr: 0,
	saved_kernel_stack: 0,
	current_thread: None
};

pub fn get() -> &'static mut PerCpuData {
	// safety: Dude trust me it's ok
	unsafe {
		&mut PER_CPU_SINGLE_CORE
	}
}

pub unsafe fn get_base() -> usize {
	((&PER_CPU_SINGLE_CORE) as *const PerCpuData) as usize
}