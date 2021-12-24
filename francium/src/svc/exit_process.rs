use crate::aarch64::context::ExceptionContext;
use crate::scheduler;

pub fn svc_exit_process(_exc: &mut ExceptionContext) {
	scheduler::terminate_current_process();
}