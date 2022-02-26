use crate::arch::context::ExceptionContext;
use crate::scheduler;

#[cfg(target_arch = "aarch64")]
pub fn svc_get_process_id(exc: &mut ExceptionContext) {
	exc.regs[0] = scheduler::get_current_process().lock().id;
}

#[cfg(target_arch = "x86_64")]
pub fn svc_get_process_id(exc: &mut ExceptionContext) {
	unimplemented!();
}