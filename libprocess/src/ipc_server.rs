use crate::syscalls;
use common::Handle;
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
    should_stop: bool,
    sessions: HashMap<Handle, Arc<dyn IPCSession>>,
    new_session_event: Handle
}

impl std::fmt::Debug for ServerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("ServerImpl")
    }
}

impl ServerImpl {
    pub fn new(port: Handle) -> ServerImpl {
        let new_session_event = syscalls::create_event().unwrap();

        ServerImpl {
            handles: vec![port, new_session_event],
            should_stop: false,
            sessions: HashMap::new(),
            new_session_event: new_session_event
        }
    }

    pub fn register_session(&mut self, h: Handle, s: Arc<dyn IPCSession>) {
        self.sessions.insert(h, s);
        self.handles.push(h);
        syscalls::signal_event(self.new_session_event).unwrap();
    }
}

pub trait IPCServer {
    fn get_server_impl<'a>(self: &'a Arc<Self>) -> MutexGuard<'a, ServerImpl>;
    fn accept_main_session_in_trait(self: &Arc<Self>) -> Arc<dyn IPCSession>;

    fn process_forever(self: Arc<Self>) {
        loop {
            let mut ipc_buffer: [u8; 128] = [0; 128];

            let server = self.get_server_impl();
            /* ugh i hate this but w/e */
            let (index, mut ipc_buffer) = tokio::task::block_in_place(|| {
                let copied_handles = server.handles.clone();
                drop(server);

                let i = syscalls::ipc_receive(&copied_handles, &mut ipc_buffer).unwrap();
                (i, ipc_buffer)
            });

            let mut server = self.get_server_impl();
            if index == 0 {
                // server handle is signalled!
                let new_session = syscalls::ipc_accept(server.handles[0]).unwrap();
                let session = self.accept_main_session_in_trait();
                server.sessions.insert(new_session, session);
                server.handles.push(new_session);
                drop(server);
            } else if index == 1 {
                // new session, do nothing
                syscalls::clear_event(server.new_session_event).unwrap();
                drop(server);
            } else {
                // a client has a message for us!
                // todo: maybe move message into here?

                let handle = server.handles[index];
                let session = server.sessions[&handle].clone();

                drop(server);
                session.process(handle, &mut ipc_buffer);
            }

            let server = self.get_server_impl();
            if server.should_stop {
                break;
            }
        }
    }
}

pub trait IPCSession: Send + Sync {
    fn process(self: Arc<Self>, h: Handle, ipc_buffer: &mut [u8]);
}
