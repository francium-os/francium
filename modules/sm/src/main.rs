use std::sync::Mutex;
use hashbrown::HashMap;

use async_broadcast::broadcast;

use process::syscalls;
use process::Handle;
use process::os_error::OSResult;
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::*;

include!(concat!(env!("OUT_DIR"), "/sm_server_impl.rs"));

struct SMServerStruct {
	server_ports: Mutex<HashMap<u64, Handle>>,
	server_waiters: Mutex<HashMap<u64, (async_broadcast::Sender<Handle>, async_broadcast::Receiver<Handle>)>>
}

impl SMServerStruct {
	async fn get_service_handle(&self, tag: u64) -> OSResult<TranslateMoveHandle> {
		println!("Got tag: {:x}", tag);

		let server_port = {
			self.server_ports.lock().unwrap().get(&tag).map(|x| *x)
		};

		let server_port = match server_port {
			Some(x) => x,
			None => {
				println!("waiting for port {:x}", tag);
				
				let mut waiter = {
					let mut server_waiters_locked = self.server_waiters.lock().unwrap();

					match server_waiters_locked.get(&tag) {
						Some(ref x) => {
							x.1.clone()
						},
						None => {
							let (s, r) = broadcast(1);
							server_waiters_locked.insert(tag, (s,r.clone()));
							r
						}
					}
				};

				waiter.recv().await.unwrap()
			}
		};

		let client_session = syscalls::connect_to_port_handle(server_port)?;
		Ok(TranslateMoveHandle(client_session))
	}

	async fn register_port(&self, tag: u64, port_handle: TranslateCopyHandle) -> OSResult<()> {
		println!("registering port {:x}", tag);
		self.server_ports.lock().unwrap().insert(tag, port_handle.0);

		let new_tag = self.server_waiters.lock().unwrap().remove(&tag);
		if let Some((send, _recv)) = new_tag {
			println!("signalling port {:x}", tag);
			send.broadcast(port_handle.0).await.unwrap();
		}

		Ok(())
	}
}

#[tokio::main]
async fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let server = ServerImpl::new(SMServerStruct{ server_ports: Mutex::new(HashMap::new()), server_waiters: Mutex::new(HashMap::new()) }, port);

	server.process_forever().await;

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}