use common::{Handle, INVALID_HANDLE};
use crate::syscalls;
use smallvec::SmallVec;
use alloc::boxed::Box;
use core::sync::atomic::{Ordering, AtomicBool};
use crate::ipc::message::IPC_BUFFER;
use alloc::sync::Arc;

#[async_trait::async_trait]
pub trait IPCServer {
	async fn process(self: std::sync::Arc<Self>, h: Handle);
}

pub struct ServerImpl<T> {
	handles: SmallVec<[Handle; 2]>,
	should_stop: AtomicBool,
	pub server: Arc<T>
}

impl<T: IPCServer + Send + Sync + 'static> ServerImpl<T> {
	pub fn new(t: T, port: Handle) -> ServerImpl<T> {
		ServerImpl {
			handles: SmallVec::from_buf_and_len([port, INVALID_HANDLE], 1),
			should_stop: AtomicBool::new(false),
			server: Arc::new(t)
		}
	}

	// async: this needs to _move_ self
	pub async fn process_forever(mut self)
	{
		loop {
			let index = unsafe { syscalls::ipc_receive(&self.handles, &mut IPC_BUFFER).unwrap() };
			if index == 0 {
				// server handle is signalled!
				let new_session = syscalls::ipc_accept(self.handles[0]).unwrap();
				self.handles.push(new_session);
				println!("IPC: accepted new session");
			} else {
				// a client has a message for us!
				// todo: maybe move message into here?
				let handle = self.handles[index];
				let server = self.server.clone();
				server.process(handle).await;
			}

			if self.should_stop.load(Ordering::Acquire) {
				break;
			}
		}
	}
}