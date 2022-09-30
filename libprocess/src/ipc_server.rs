use crate::{Handle, INVALID_HANDLE};
use crate::syscalls;
use smallvec::SmallVec;

pub trait IPCServer {
	fn new() -> Self;
	fn process(&self, h: Handle);
}

// https://stackoverflow.com/questions/29978133/capturing-a-trait-in-a-struct-that-is-only-used-in-the-implementation
pub struct ServerImpl<T: IPCServer> {
	handles: SmallVec<[Handle; 2]>,
	server: T
}

impl<T: IPCServer> ServerImpl<T> {
	pub fn new(port: Handle) -> ServerImpl<T> {
		ServerImpl {
			handles: SmallVec::from_buf_and_len([port, INVALID_HANDLE], 1),
			server: T::new()
		}
	}

	pub fn process(&mut self) -> bool {
		let index = syscalls::ipc_receive(&self.handles).unwrap();
		if index == 0 {
			// server handle
			let new_session = syscalls::ipc_accept(self.handles[0]).unwrap();
			self.handles.push(new_session);
			println!("IPC: accepted new session");

			true
		} else {
			// a client has a message for us!
			self.server.process(self.handles[index]);
			true
		}
	}
}