use crate::syscalls;
use alloc::sync::Arc;
use common::{Handle, INVALID_HANDLE};
use core::sync::atomic::{AtomicBool, Ordering};
use smallvec::SmallVec;
use tokio;

pub trait IPCServer {
    fn process(self: std::sync::Arc<Self>, h: Handle, ipc_buffer: &mut [u8]);
}

pub struct ServerImpl<T> {
    handles: SmallVec<[Handle; 2]>,
    should_stop: AtomicBool,
    pub server: Arc<T>,
}

impl<T: IPCServer + Send + Sync + 'static> ServerImpl<T> {
    pub fn new(t: T, port: Handle) -> ServerImpl<T> {
        ServerImpl {
            handles: SmallVec::from_buf_and_len([port, INVALID_HANDLE], 1),
            should_stop: AtomicBool::new(false),
            server: Arc::new(t),
        }
    }

    // async: this needs to _move_ self
    pub async fn process_forever(mut self) {
        loop {
            let mut ipc_buffer: [u8; 128] = [0; 128];

            /* ugh i hate this but w/e */
            let handles_copy = self.handles.clone();

            let (index, mut ipc_buffer) = tokio::task::spawn_blocking(move || {
                let i = syscalls::ipc_receive(&handles_copy, &mut ipc_buffer).unwrap();
                (i, ipc_buffer)
            }).await.unwrap();

            if index == 0 {
                // server handle is signalled!
                let new_session = syscalls::ipc_accept(self.handles[0]).unwrap();
                self.handles.push(new_session);
            } else {
                // a client has a message for us!
                // todo: maybe move message into here?
                let handle = self.handles[index];
                let server = self.server.clone();
                server.process(handle, &mut ipc_buffer);
            }

            if self.should_stop.load(Ordering::Acquire) {
                break;
            }
        }
    }
}
