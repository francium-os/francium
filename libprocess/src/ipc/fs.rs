use spin::Mutex;
use common::Handle;
use common::os_error::OSResult;
use ipc_gen::ipc_server;
use common::ipc::*;

static FS_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

fn get_handle_for_fs() -> Handle {
	let mut locked = FS_HANDLE.lock();
	match *locked {
		Some(x) => x,
		None => {
			let handle = crate::ipc::sm::get_service_handle(crate::syscalls::make_tag("fs")).unwrap().0;
			*locked = Some(handle);
			handle
		}
	}
}

#[ipc_server(get_handle_for_fs)]
trait FSServer {
	#[ipc_method_id = 0]
	fn stop(&self);

	#[ipc_method_id = 1]
	fn test(&self) -> OSResult<TranslateMoveHandle>;
}
