use tracing::{event, Level};

use crate::scheduler;
use crate::waitable::Waiter;
use common::os_error::{Module, Reason, ResultCode, RESULT_OK};
use core::sync::atomic::{AtomicU32, Ordering};
use hashbrown::{hash_map::Entry, HashMap};
use spin::Mutex;

lazy_static! {
    static ref FUTEX_TABLE: Mutex<HashMap<usize, Waiter>> = Mutex::new(HashMap::new());
}

pub fn svc_futex_wait(addr: usize, expected: u32, _timeout_ns: usize) -> ResultCode {
    event!(
        Level::TRACE,
        svc_name = "futex_wait",
        addr = addr,
        expected = expected,
        timeout = _timeout_ns
    );
    // TODO: uhhh
    let futex_valid = unsafe { (*(addr as *mut AtomicU32)).load(Ordering::SeqCst) == expected };

    if futex_valid {
        {
            let mut table_lock = FUTEX_TABLE.lock();
            let waiter = match table_lock.entry(addr) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(Waiter::new()),
            };
            waiter.post_wait(0);
        }
        scheduler::suspend_current_thread();

        RESULT_OK
    } else {
        ResultCode::new(Module::Kernel, Reason::TryAgain)
    }
}

pub fn svc_futex_wake(addr: usize) -> ResultCode {
    event!(
        Level::TRACE,
        svc_name = "futex_wake",
        addr = addr
    );

    match FUTEX_TABLE.lock().get(&addr) {
        Some(x) => {
            x.signal_all();
        }
        None => {
            // TODO: what do we do? currently nothing
            //println!("No futex!");
        }
    }

    RESULT_OK
}
