use spin::Mutex;
use crate::Handle;
use crate::syscalls;
use common::os_error::OSResult;
use ipc_gen::ipc_server;
use crate::ipc::message::TranslateHandle;

static SM_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

fn get_handle_for_sm() -> Handle {
	let mut locked = SM_HANDLE.lock();
	match *locked {
		Some(x) => x,
		None => {
			let handle = syscalls::connect_to_port("sm").unwrap();
			*locked = Some(handle);
			handle
		}
	}
}

#[ipc_server(get_handle_for_sm)]
trait SMServer {
	#[ipc_method_id = 0]
	fn stop(&self);

	#[ipc_method_id = 1]
	//#[copy_handles(return_value)]
	fn get_service_handle(&self, tag: u64) -> OSResult<TranslateHandle>;

	#[ipc_method_id = 2]
	fn register_port(&self, tag: u64, port_handle: TranslateHandle) -> OSResult<()>;
}
