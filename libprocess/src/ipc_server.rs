use common::{Handle, INVALID_HANDLE};
use crate::syscalls;
use smallvec::SmallVec;

pub trait IPCServer {
	fn process(&mut self, h: Handle);
}

pub struct ServerImpl<T> where T: IPCServer {
	handles: SmallVec<[Handle; 2]>,
	pub server: T
}

impl<T: IPCServer> ServerImpl<T> {
	pub fn new(t: T, port: Handle) -> ServerImpl<T> {
		ServerImpl {
			handles: SmallVec::from_buf_and_len([port, INVALID_HANDLE], 1),
			server: t
		}
	}

	pub fn process(&mut self) -> bool
	{
		let index = syscalls::ipc_receive(&self.handles).unwrap();
		if index == 0 {
			// server handle is signalled!
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