use spin::Mutex;
use crate::Handle;
use common::os_error::OSResult;
use ipc_gen::ipc_server;
use crate::ipc::message::TranslateHandle;

static FS_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

fn get_handle_for_fs() -> Handle {
	let mut locked = FS_HANDLE.lock();
	match *locked {
		Some(x) => x,
		None => {
			let handle = *crate::ipc::sm::get_service_handle(crate::syscalls::make_tag("fs")).unwrap();
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
	fn test(&self) -> OSResult<TranslateHandle>;
}
