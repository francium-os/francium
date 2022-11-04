use hashbrown::HashMap;

use async_broadcast::broadcast;

use process::println;
use process::syscalls;
use process::Handle;
use process::os_error::OSResult;
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::*;

include!(concat!(env!("OUT_DIR"), "/sm_server_impl.rs"));

struct SMServerStruct {
	server_ports: HashMap<u64, Handle>,
	server_waiters: HashMap<u64, (async_broadcast::Sender<Handle>, async_broadcast::Receiver<Handle>)>
}

impl SMServerStruct {
	fn stop(&self) {
		unimplemented!();
	}

	async fn get_service_handle(&mut self, tag: u64) -> OSResult<TranslateMoveHandle> {
		println!("Got tag: {:x}", tag);

		let server_port = {
			self.server_ports.get(&tag).map(|x| *x)
		};

		let server_port = match server_port {
			Some(x) => x,
			None => {
				println!("waiting for port {:x}", tag);
				let mut waiter = match self.server_waiters.get(&tag) {
					Some(ref x) => {
						x.1.clone()
					},
					None => {
						let (s, r) = broadcast(1);
						self.server_waiters.insert(tag, (s,r.clone()));
						r
					}
				};

				waiter.recv().await.unwrap()
			}
		};

		let client_session = syscalls::connect_to_port_handle(server_port)?;
		Ok(TranslateMoveHandle(client_session))
	}

	async fn register_port(&mut self, tag: u64, port_handle: TranslateCopyHandle) -> OSResult<()> {
		println!("registering port {:x}", tag);
		self.server_ports.insert(tag, port_handle.0);

		if let Some((send, _recv)) = self.server_waiters.remove(&tag) {
			println!("signalling port {:x}", tag);
			send.broadcast(port_handle.0).await.unwrap();
		}

		Ok(())
	}
}

fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let server = ServerImpl::new(SMServerStruct{ server_ports: HashMap::new(), server_waiters: HashMap::new() }, port);

	futures::executor::block_on(server.process_forever());

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}