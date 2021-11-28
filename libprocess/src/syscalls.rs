use crate::{Handle,ResultCode,RESULT_OK};
use crate::os_error::{OSError,result_to_error};
use core::cmp::min;

extern "C" {
	pub fn syscall_debug_output(s: *const u8, len: usize) -> ResultCode;
	pub fn syscall_create_port(tag: u64, handle_out: *mut Handle) -> ResultCode;
	pub fn syscall_connect_to_port(tag: u64, handle_out: *mut Handle) -> ResultCode;
	pub fn syscall_exit_process() -> !;
	pub fn syscall_close_handle(h: Handle) -> ResultCode;
	pub fn syscall_ipc_request(session_handle: Handle) -> ResultCode;
	pub fn syscall_ipc_reply(session_handle: Handle) -> ResultCode;
	pub fn syscall_ipc_receive(session_handle: Handle) -> ResultCode;
}

pub fn print(s: &str) {
	unsafe {
		syscall_debug_output(s.as_bytes().as_ptr(), s.len());
	}
}

fn make_tag(s: &str) -> u64 {
	let tag_bytes = s.as_bytes();
	let length = min(8, tag_bytes.len());
	let mut tag_bytes_padded: [u8; 8] = [0; 8];
	tag_bytes_padded[0..length].copy_from_slice(tag_bytes);
	
	u64::from_be_bytes(tag_bytes_padded)
}

pub fn create_port(s: &str) -> Result<Handle, OSError> {
	let mut handle_out = Handle(0);
	unsafe {
		let res = syscall_create_port(make_tag(s), &mut handle_out);
		if res == RESULT_OK {
			Ok(handle_out)
		} else {
			Err(result_to_error(res))
		}
	}
}

pub fn connect_to_port(s: &str) -> Result<Handle, OSError> {
	let mut handle_out = Handle(0);
	unsafe {
		let res = syscall_connect_to_port(make_tag(s), &mut handle_out);
		if res == RESULT_OK {
			Ok(handle_out)
		} else {
			Err(result_to_error(res))
		}
	}
}

pub fn close_handle(h: Handle) -> Result<(), OSError> {
	unsafe {
		let res = syscall_close_handle(h);
		if res == RESULT_OK {
			Ok(())
		} else {
			Err(result_to_error(res))
		}
	}
}

pub fn exit_process() -> ! {
	unsafe {
		syscall_exit_process();
	}
}

pub fn ipc_request(session_handle: Handle) -> Result<(), OSError> {
	unsafe {
		let res = syscall_ipc_request(session_handle);
		if res == RESULT_OK {
			Ok(())
		} else {
			Err(result_to_error(res))
		}
	}
}

pub fn ipc_reply(session_handle: Handle) -> Result<(), OSError> {
	unsafe {
		let res = syscall_ipc_reply(session_handle);
		if res == RESULT_OK {
			Ok(())
		} else {
			Err(result_to_error(res))
		}
	}
}

pub fn ipc_receive(session_handle: Handle) -> Result<(), OSError> {
	unsafe {
		let res = syscall_ipc_receive(session_handle);
		if res == RESULT_OK {
			Ok(())
		} else {
			Err(result_to_error(res))
		}
	}
}