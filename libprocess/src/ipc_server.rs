use crate::syscalls;
use common::Handle;
use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, MutexGuard};
use tokio;

#[macro_export]
macro_rules! define_server {
    ($struct_name: ident {
        $($manual_fields:tt)*
    }) => {
        struct $struct_name {
            __server_impl: Mutex<ServerImpl>,
            $($manual_fields)*
        }
    }
}

#[macro_export]
macro_rules! define_session {
    ($struct_name: ident {
        $($manual_fields:tt)*
    },
    $server_type:tt) => {
        struct $struct_name {
            __server: Arc<$server_type>,
            $($manual_fields)*
        }
    }
}

pub struct ServerImpl {
    handles: Vec<Handle>,
    should_stop: AtomicBool,
    sessions: HashMap<Handle, Arc<dyn IPCSession>>,
}

impl ServerImpl {
    pub fn new(port: Handle) -> ServerImpl {
        ServerImpl {
            handles: vec![port],
            should_stop: AtomicBool::new(false),
            sessions: HashMap::new(),
        }
    }

    pub fn register_session(&mut self, h: Handle, s: Arc<dyn IPCSession>) {
        self.sessions.insert(h, s);
        self.handles.push(h)
    }
}

#[async_trait::async_trait]
pub trait IPCServer {
    fn get_server_impl<'a>(self: &'a Arc<Self>) -> MutexGuard<'a, ServerImpl>;
    fn accept_main_session_in_trait(self: &Arc<Self>) -> Arc<dyn IPCSession>;

    async fn process_forever(self: Arc<Self>) {
        loop {
            let mut ipc_buffer: [u8; 128] = [0; 128];

            let mut server = self.get_server_impl();
            /* ugh i hate this but w/e */
            let (index, mut ipc_buffer) = tokio::task::block_in_place(|| {
                let i = syscalls::ipc_receive(&server.handles, &mut ipc_buffer).unwrap();
                (i, ipc_buffer)
            });

            if index == 0 {
                // server handle is signalled!
                let new_session = syscalls::ipc_accept(server.handles[0]).unwrap();
                let session = self.accept_main_session_in_trait();
                server.sessions.insert(new_session, session);
                server.handles.push(new_session);
            } else {
                // a client has a message for us!
                // todo: maybe move message into here?

                let handle = server.handles[index];
                let session = server.sessions[&handle].clone();

                session.process(handle, &mut ipc_buffer);
            }

            if server.should_stop.load(Ordering::Acquire) {
                break;
            }
        }
    }
}

pub trait IPCSession: Send + Sync {
    fn process(self: Arc<Self>, h: Handle, ipc_buffer: &mut [u8]);
}
