use tracing::{event, Level};

use alloc::boxed::Box;
use crate::timer;
use crate::scheduler;

pub fn svc_sleep_ns(ns: u64) {
	event!(Level::DEBUG, svc_name = "svc_sleep_ns", delay = ns);

	let thread = scheduler::get_current_thread();

	timer::register_timer(ns, Box::new(move || {
		scheduler::wake_thread(thread, 0xffffffffffffffff);
	}));

	scheduler::suspend_current_thread();
}
