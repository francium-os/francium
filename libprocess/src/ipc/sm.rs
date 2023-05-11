use crate::syscalls;
use common::ipc::*;
use crate::os_error::OSResult;
use common::Handle;
use spin::Mutex;

static SM_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

fn get_handle_for_sm() -> Handle {
    let mut locked = SM_HANDLE.lock();
    match *locked {
        Some(x) => x,
        None => {
            let handle = syscalls::connect_to_named_port("sm").unwrap();
            *locked = Some(handle);
            handle
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/sm_client_impl.rs"));
