use alloc::sync::Arc;
use crate::process::Thread;
use crate::arch;

pub struct PerCpuData {
	pub per_cpu_ptr: usize,
	pub saved_kernel_stack: usize,
	pub current_thread: Option<Arc<Thread>>
}

pub fn get() -> &'static mut PerCpuData {
	unsafe {
		(arch::get_per_cpu_base() as *mut PerCpuData).as_mut().unwrap()
	}
}

pub unsafe fn get_base() -> usize {
	arch::get_per_cpu_base()
}

pub fn get_current_thread() -> Arc<Thread> {
	get().current_thread.as_ref().unwrap().clone()
}

pub fn set_current_thread(a: Arc<Thread>) {
	get().current_thread = Some(a);
}