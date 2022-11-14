use crate::scheduler;

pub fn svc_get_process_id() -> usize {
	scheduler::get_current_process().lock().id
}

pub fn svc_get_thread_id() -> usize {
	scheduler::get_current_thread().id
}