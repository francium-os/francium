use crate::Handle;
use crate::ipc_common::IPC_BUFFER;
use crate::syscalls;
//use common::os_error::OSError;

pub fn try_make_request(h: Handle) {
	unsafe {
		IPC_BUFFER[0] = 0;
	}

	syscalls::ipc_request(h).unwrap();

	unsafe {
		println!("owo? {:x}", IPC_BUFFER[0]);
	}

	//OSError::from_result_code(IPC_BUFFER[1])
}