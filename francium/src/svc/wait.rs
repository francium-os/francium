use crate::waitable;
use common::os_error::{ResultCode, RESULT_OK};
use tracing::{event, Level};

pub fn svc_wait_one(handle: u32) -> ResultCode {
    event!(Level::TRACE, svc_name = "wait_one", handle = handle);
    waitable::wait_handles(&[handle]);
    RESULT_OK
}

const MAX_HANDLES: usize = 128;
pub fn svc_wait_many(handles_ptr: *const u32, handle_count: usize) -> (ResultCode, usize) {
    event!(
        Level::TRACE,
        svc_name = "svc_wait_many",
        handles_ptr = handles_ptr as usize,
        handle_count = handle_count
    );

    let mut handles: [u32; MAX_HANDLES] = [0xffffffff; MAX_HANDLES];

    unsafe {
        core::ptr::copy_nonoverlapping(handles_ptr, &mut handles as *mut u32, handle_count);
    }

    let index = waitable::wait_handles(&handles[..handle_count]);

    (RESULT_OK, index)
}
