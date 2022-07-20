use crate::scheduler;

pub fn svc_exit_process() {
	scheduler::terminate_current_process();
}