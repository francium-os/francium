use crate::{Handle, INVALID_HANDLE};
use crate::ipc_common::IPC_BUFFER;
use crate::syscalls;
use smallvec::SmallVec;
use core::marker::PhantomData;

pub trait IPCServer {
	fn handle(h: Handle);
}

// https://stackoverflow.com/questions/29978133/capturing-a-trait-in-a-struct-that-is-only-used-in-the-implementation
pub struct ServerImpl<T: IPCServer> {
	handles: SmallVec<[Handle; 2]>,
	_marker: PhantomData<T>
}

impl<T: IPCServer> ServerImpl<T> {
	pub fn new(port: Handle) -> ServerImpl<T> {
		ServerImpl {
			handles: SmallVec::from_buf_and_len([port, INVALID_HANDLE], 1),
			_marker: PhantomData
		}
	}

	pub fn process(&mut self) -> bool {
		let index = syscalls::ipc_receive(&self.handles).unwrap();
		println!("[S] Got index? {:?}", index);
		if index == 0 {
			// server handle
			let new_session = syscalls::ipc_accept(self.handles[0]).unwrap();
			self.handles.push(new_session);
			println!("IPC: accepted new session");

			true
		} else {
			// a client has a message for us!
			unsafe {
				println!("owo: {:x}", IPC_BUFFER[0]);
				IPC_BUFFER[0] = 0xaaaaaaaa;
			}
			T::handle(self.handles[index]);
			true
		}
	}
}