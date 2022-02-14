use crate::ExceptionContext;
use crate::scheduler;

pub fn svc_get_process_id(exc: &mut ExceptionContext) {
	exc.regs[0] = scheduler::get_current_process().lock().id;
}