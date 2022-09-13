use crate::{Handle, INVALID_HANDLE};
use crate::ipc_common::IPC_BUFFER;
use crate::syscalls;

pub fn try_make_request(h: Handle) {
	unsafe {
		IPC_BUFFER[0] = 0xaa;
	}

	syscalls::ipc_request(h).unwrap();
}