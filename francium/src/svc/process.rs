use tracing::{event, Level};

use crate::init;
use crate::process::Thread;
use crate::scheduler;
use common::os_error::ResultCode;

pub fn svc_get_process_id() -> usize {
    event!(Level::TRACE, svc_name = "get_process_id");

    scheduler::get_current_process().lock().id
}

pub fn svc_get_thread_id() -> usize {
    // Wayyy too noisy.
    //event!(Level::TRACE, svc_name = "get_thread_id");
    scheduler::get_current_thread().id
}

pub fn svc_create_thread(entry_point: usize, stack_top: usize) -> (ResultCode, u32) {
    event!(
        Level::TRACE,
        svc_name = "create_thread",
        entry_point = entry_point,
        stack_top = stack_top
    );

    let process = scheduler::get_current_process();
    let new_thread = Thread::new(process);

    init::setup_thread_context(&new_thread, entry_point, stack_top, false);
    let tid = new_thread.id;
    scheduler::register_thread(new_thread);

    // TODO: This is meant to return a thread handle!
    (ResultCode(0), tid as u32)
}
