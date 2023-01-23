use crate::scheduler;
use tracing::{event, Level};

pub fn svc_exit_process() {
    event!(Level::TRACE, svc_name = "exit_process");
    scheduler::terminate_current_process();
}
