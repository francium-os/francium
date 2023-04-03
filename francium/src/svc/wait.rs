use tracing::{event, Level};
use common::os_error::{ResultCode, RESULT_OK};
use crate::waitable;

pub fn svc_wait_one(handle: u32) -> ResultCode {
    event!(Level::TRACE, svc_name = "wait_one", handle = handle);
    waitable::wait_handles(&[handle]);
    RESULT_OK
}