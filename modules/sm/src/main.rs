use std::sync::atomic::{AtomicBool, Ordering};
use hashbrown::HashMap;

use process::println;
use process::syscalls;
use process::{Handle, INVALID_HANDLE};
use process::os_error::{OSError, OSResult, Module, Reason};
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::*;
use process::ipc::sm::SMServer;

include!(concat!(env!("OUT_DIR"), "/sm_server_impl.rs"));

struct SMServerStruct {
	server_ports: HashMap<u64, Handle>
}

impl SMServerStruct {
	async fn get_service_handle(&self, tag: u64) -> OSResult<TranslateMoveHandle> {
		println!("Got tag: {:x}", tag);
		let server_port = self.server_ports.get(&tag).ok_or(OSError::new(Module::SM, Reason::NotFound))?;
		let client_session = syscalls::connect_to_port_handle(*server_port)?;
		Ok(TranslateMoveHandle(client_session))
	}

	fn register_port(&mut self, tag: u64, port_handle: TranslateCopyHandle) -> OSResult<()> {
		println!("registering port {:x}", tag);
		self.server_ports.insert(tag, port_handle.0);
		Ok(())
	}
}

fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let mut server = ServerImpl::new(SMServerStruct{ server_ports: HashMap::new() }, port);

	let exc = ::pasts::Executor::default();
	exc.spawn(server.process_forever());

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}
