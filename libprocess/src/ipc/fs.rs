//use common::ipc::*;
use crate::os_error::OSResult;
use common::Handle;
use spin::Mutex;

static FS_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

fn get_handle_for_fs() -> Handle {
    let mut locked = FS_HANDLE.lock();
    match *locked {
        Some(x) => x,
        None => {
            let handle = crate::ipc::sm::get_service_handle(crate::syscalls::make_tag("fs"))
                .unwrap()
                .0;
            *locked = Some(handle);
            handle
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/fs_client_impl.rs"));
