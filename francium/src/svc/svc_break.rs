use crate::arch::context::ExceptionContext;

pub fn svc_break(_: &mut ExceptionContext) {
	panic!("svcBreak called!");
}