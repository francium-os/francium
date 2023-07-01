use async_trait::async_trait;
use crate::syscalls;
use crate::Handle;
use std::sync::{Arc, Mutex};
use std::sync::MutexGuard;
use std::sync::atomic::Ordering;
use std::collections::HashMap;

pub struct ServerImpl<'a, T> where T: IPCServer<'a> + ?Sized + 'a {
	pub sessions: HashMap<Handle, Box<dyn IPCSession<'a, Server = T> + 'a>>,
    handles: Vec<Handle>,
    should_stop: bool
}

impl<'a, T> ServerImpl<'a, T> where T: IPCServer<'a> + ?Sized + 'a {
    pub fn new() -> ServerImpl<'a, T> {
        ServerImpl {
            sessions: HashMap::new(),
            handles: Vec::new(),
            should_stop: false
        }
    }

    pub fn register_session(self: &ServerImpl<'a, T>, handle: Handle, session: Box<dyn IPCSession<'a, Server = T> + 'a>) {

    }
}

#[async_trait]
pub trait IPCServer<'a> {
    fn get_server_impl<'m, 'r>(self: &'r Arc<Self>) -> MutexGuard<'m, ServerImpl<'a, Self>> where 'r: 'm;
    fn accept_main_session_in_trait(self: &Arc<Self>) -> Box<dyn IPCSession<'a, Server = Self>>;
	fn process_server(self: Arc<Self>, handle: Handle, ipc_buffer: &mut [u8]);

	async fn process_forever(self: Arc<Self>) where Self: 'a {
        loop {
            let mut ipc_buffer: [u8; 128] = [0; 128];

            let (index, mut ipc_buffer) = tokio::task::block_in_place(|| {
                let mut server = self.clone();
                let server_impl = server.get_server_impl();

                let i = syscalls::ipc_receive(&server_impl.handles, &mut ipc_buffer).unwrap();
               (i, ipc_buffer)
            });

            if index == 0 {
                // server handle is signalled!
                let server = self.clone();
                let mut server_impl = server.get_server_impl();

                let new_session = syscalls::ipc_accept(server_impl.handles[0]).unwrap();
                let session = self.accept_main_session_in_trait();
                server_impl.sessions.insert(new_session, session);
                server_impl.handles.push(new_session);
            } else {
                // a client has a message for us!
                // todo: maybe move message into here?

                let server = self.clone();
                let server_impl = server.get_server_impl();

                let handle = server_impl.handles[index];
                drop(server_impl);
                drop(server);

                self.clone().process_server(handle, &mut ipc_buffer);
            }

            let server_impl = self.get_server_impl();
            if server_impl.should_stop {
                break;
            }
        }
    }
}

pub trait IPCSession<'a>: Send + Sync {
    type Server;
    fn process_session(&self, server: Arc<Self::Server>, handle: Handle, ipc_buffer: &mut [u8]);
}

    
