use crate::Handle;
use common::os_error::OSResult;
use ipc_gen::{ipc_server, /*ipc_method, copy_handles, move_handles*/};

fn get_handle_for_sm() -> Handle {
	unimplemented!()
}

#[ipc_server]
trait SMServer {
	#[ipc_method_id = 1]
	//#[copy_handles(return_value)]
	fn get_service_handle(&self, tag: u64) -> OSResult<Handle>;
}
