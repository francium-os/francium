use spin::Mutex;
use crate::Handle;
use crate::syscalls;
use common::os_error::OSResult;
use ipc_gen::ipc_server;

static FS_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

#[inline(never)]
fn get_handle_for_fs() -> Handle {
	let mut locked = FS_HANDLE.lock();
	match *locked {
		Some(x) => x,
		None => {
			let handle = syscalls::connect_to_port("fs").unwrap();
			*locked = Some(handle);
			handle
		}
	}
}

#[ipc_server]
trait FSServer {
}
