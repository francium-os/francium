use crate::aarch64::context::ExceptionContext;
use crate::scheduler;

pub fn svc_close_handle(exc: &mut ExceptionContext) {
	// todo: proper thread locals, etc etc.
	let p_ = scheduler::get_current_process();
	let mut p = p_.lock();

	//exc.regs[0] = 0;
	exc.regs[0] = p.handle_table.close(exc.regs[0] as u32) as usize;
}