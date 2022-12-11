use tracing::{event, Level};

use crate::scheduler;
use common::os_error::ResultCode;

pub fn svc_close_handle(handle: u32) -> ResultCode {
    event!(Level::DEBUG, svc_name = "close_handle", handle = handle);

    // todo: proper thread locals, etc etc.
    let p_ = scheduler::get_current_process();
    let mut p = p_.lock();

    p.handle_table.close(handle)
}
