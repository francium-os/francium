#![no_std]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
use hashbrown::HashMap;

use process::println;
use process::syscalls;
use process::{Handle, INVALID_HANDLE};
use process::os_error::{OSError, OSResult, Module, Reason};
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::*;
use process::ipc::sm::SMServer;

struct SMServerStruct {
	should_stop: AtomicBool,
	server_ports: HashMap<u64, Handle>
}

impl IPCServer for SMServerStruct {
	fn process(&mut self, h: Handle) {
		SMServer::process(self, h)
	}
}

impl SMServer for SMServerStruct {
	fn stop(&self) {
		println!("SM stopping!");
		self.should_stop.store(true, Ordering::Release);
	}

	fn get_service_handle(&self, tag: u64) -> OSResult<TranslateMoveHandle> {
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
	let mut server = ServerImpl::new(SMServerStruct{ should_stop: AtomicBool::new(false), server_ports: HashMap::new() }, port);

	while server.process() {
		if server.server.should_stop.load(Ordering::Acquire) {
			break
		}
	}

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}
