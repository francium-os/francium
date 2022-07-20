use crate::scheduler;

pub fn svc_close_handle(handle: u32) -> u32 {
	// todo: proper thread locals, etc etc.
	let p_ = scheduler::get_current_process();
	let mut p = p_.lock();

	p.handle_table.close(handle)
}