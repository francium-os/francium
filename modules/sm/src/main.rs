use hashbrown::HashMap;
use std::sync::{Arc, Mutex};

use async_broadcast::broadcast;

use process::ipc::*;
use process::ipc_server::{IPCServer, ServerImpl};
use process::os_error::OSResult;
use process::syscalls;
use process::Handle;
use process::{define_server, define_session};

include!(concat!(env!("OUT_DIR"), "/sm_server_impl.rs"));

define_server!(SMServerStruct {
    server_ports: Mutex<HashMap<u64, Handle>>,
    server_waiters: Mutex<
        HashMap<
            u64,
            (
                async_broadcast::Sender<Handle>,
                async_broadcast::Receiver<Handle>,
            ),
        >,
    >,
});

define_session!(SMSession {}, SMServerStruct);

impl SMServerStruct {
    fn accept_main_session(self: &Arc<SMServerStruct>) -> Arc<SMSession> {
        Arc::new(SMSession {
            __server: self.clone(),
        })
    }
}

impl SMSession {
    async fn get_service_handle(&self, tag: u64) -> OSResult<TranslateMoveHandle> {
        let server_port = {
            self.get_server()
                .server_ports
                .lock()
                .unwrap()
                .get(&tag)
                .map(|x| *x)
        };

        let server_port = match server_port {
            Some(x) => x,
            None => {
                let mut waiter = {
                    let server = self.get_server();
                    let mut server_waiters_locked = server.server_waiters.lock().unwrap();

                    match server_waiters_locked.get(&tag) {
                        Some(ref x) => x.1.clone(),
                        None => {
                            let (s, r) = broadcast(1);
                            server_waiters_locked.insert(tag, (s, r.clone()));
                            r
                        }
                    }
                };
                waiter.recv().await.unwrap()
            }
        };

        let client_session =
            tokio::task::block_in_place(move || syscalls::connect_to_port_handle(server_port))?;

        Ok(TranslateMoveHandle(client_session))
    }

    async fn register_port(&self, tag: u64, port_handle: TranslateCopyHandle) -> OSResult<()> {
        self.get_server()
            .server_ports
            .lock()
            .unwrap()
            .insert(tag, port_handle.0);

        let new_tag = self
            .get_server()
            .server_waiters
            .lock()
            .unwrap()
            .remove(&tag);
        if let Some((send, _recv)) = new_tag {
            send.broadcast(port_handle.0).await.unwrap();
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from sm!");

    let port = syscalls::create_port("sm").unwrap();

    let server = Arc::new(SMServerStruct {
        __server_impl: Mutex::new(ServerImpl::new(port)),
        server_ports: Mutex::new(HashMap::new()),
        server_waiters: Mutex::new(HashMap::new()),
    });

    server.process_forever();

    syscalls::close_handle(port).unwrap();
    println!("SM exiting!");

    syscalls::exit_process();
}
