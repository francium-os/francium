use tracing::{event, Level};
use crate::scheduler;

pub fn svc_exit_process() {
	event!(Level::DEBUG, svc_name = "exit_process");
	scheduler::terminate_current_process();
}