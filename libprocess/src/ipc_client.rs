use crate::Handle;
use crate::ipc_common::IPC_BUFFER;
use crate::syscalls;

pub fn try_make_request(h: Handle) {
	unsafe {
		IPC_BUFFER[0] = 0x69706320; // 'ipc '
		IPC_BUFFER[1] = 0; // TODO: pack method id/num handles/num translates
	}

	syscalls::ipc_request(h).unwrap();

	unsafe {
		println!("owo? {:x}", IPC_BUFFER[0]);
	}
}