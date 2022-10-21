use common::{Handle, INVALID_HANDLE};
use crate::syscalls;
use smallvec::SmallVec;
use alloc::boxed::Box;
use core::sync::atomic::AtomicBool;

#[async_trait::async_trait]
pub trait IPCServer {
	async fn process(&mut self, h: Handle);
}

pub struct ServerImpl<T> {
	handles: SmallVec<[Handle; 2]>,
	should_stop: AtomicBool,
	pub server: T
}

impl<T: IPCServer> ServerImpl<T> {
	pub fn new(t: T, port: Handle) -> ServerImpl<T> {
		ServerImpl {
			handles: SmallVec::from_buf_and_len([port, INVALID_HANDLE], 1),
			should_stop: AtomicBool::new(false),
			server: t
		}
	}

	// async: this needs to _move_ self
	pub async fn process_forever(mut self)
	{
		loop {
			let index = syscalls::ipc_receive(&self.handles).unwrap();
			if index == 0 {
				// server handle is signalled!
				let new_session = syscalls::ipc_accept(self.handles[0]).unwrap();
				self.handles.push(new_session);
				println!("IPC: accepted new session");
			} else {
				// a client has a message for us!
				// todo: maybe move message into here?
				self.server.process(self.handles[index]).await;
			}
		}
	}
}