use crate::syscalls;
use alloc::sync::Arc;
use common::{Handle, INVALID_HANDLE};
use core::sync::atomic::{AtomicBool, Ordering};
use smallvec::SmallVec;
use tokio;
use std::collections::HashMap;

pub trait IPCServer {
    type Session;
    type SubInterfaces;
    fn create_subinterfaces() -> Self::SubInterfaces;
    fn get_subinterface_index<T>() -> usize;

    fn accept_session_ext(self: std::sync::Arc<Self>) -> std::sync::Arc<Self::Session>;
}

pub trait IPCSession {
    fn process(self: std::sync::Arc<Self>, h: Handle, ipc_buffer: &mut [u8]);
}

pub struct ServerImpl<T> where T: IPCServer, <T as IPCServer>::Session: IPCSession {
    handles: Vec<Handle>,
    should_stop: AtomicBool,
    server: Arc<T>,
    sessions: HashMap<Handle, Arc<T::Session>>,
    sub_interfaces: T::SubInterfaces,
}

impl<T: IPCServer + Send + Sync + 'static> ServerImpl<T> where <T as IPCServer>::Session: IPCSession {
    pub fn new(t: T, port: Handle) -> ServerImpl<T> {
        ServerImpl {
            handles: vec![port],
            should_stop: AtomicBool::new(false),
            server: Arc::new(t),
            sessions: HashMap::new(),
            sub_interfaces: T::create_subinterfaces()
        }
    }

    // async: this needs to _move_ self
    pub async fn process_forever(mut self) {
        loop {
            let mut ipc_buffer: [u8; 128] = [0; 128];

            /* ugh i hate this but w/e */
            let handles_copy = self.handles.clone();

            let (index, mut ipc_buffer) = tokio::task::block_in_place(move || {
                let i = syscalls::ipc_receive(&handles_copy, &mut ipc_buffer).unwrap();
                (i, ipc_buffer)
            });

            if index == 0 {
                // server handle is signalled!
                let new_session = syscalls::ipc_accept(self.handles[0]).unwrap();
                let server = self.server.clone();
                self.sessions.insert(new_session, server.accept_session_ext());
                self.handles.push(new_session);
            } else {
                // a client has a message for us!
                // todo: maybe move message into here?

                let handle = self.handles[index];
                let server = self.server.clone();
                let session = self.sessions[&handle].clone();

                session.process(handle, &mut ipc_buffer);
            }

            if self.should_stop.load(Ordering::Acquire) {
                break;
            }
        }
    }
}
