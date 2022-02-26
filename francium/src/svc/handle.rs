use crate::arch::context::ExceptionContext;
use crate::scheduler;

#[cfg(target_arch = "aarch64")]
pub fn svc_close_handle(exc: &mut ExceptionContext) {
	// todo: proper thread locals, etc etc.
	let p_ = scheduler::get_current_process();
	let mut p = p_.lock();

	exc.regs[0] = p.handle_table.close(exc.regs[0] as u32) as usize;
}

#[cfg(target_arch = "x86_64")]
pub fn svc_close_handle(exc: &mut ExceptionContext) {
	unimplemented!();
}