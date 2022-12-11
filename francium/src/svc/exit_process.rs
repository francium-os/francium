use crate::scheduler;
use tracing::{event, Level};

pub fn svc_exit_process() {
    event!(Level::DEBUG, svc_name = "exit_process");
    scheduler::terminate_current_process();
}
