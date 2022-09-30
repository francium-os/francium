#![no_std]
#![feature(default_alloc_error_handler)]

use process::println;
use process::syscalls;
use process::{Handle, INVALID_HANDLE};
use process::os_error::OSResult;
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::sm::SMServer;

struct SMServerStruct {}

impl IPCServer for SMServerStruct {
	fn process(&self, h: Handle) {
		SMServer::process(self, h)
	}
}

impl SMServer for SMServerStruct {
	fn get_service_handle(&self, tag: u64) -> OSResult<Handle> {
		println!("Got tag: {:x}", tag);
		Ok(INVALID_HANDLE)
	}
}

fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let mut server = ServerImpl::new(SMServerStruct{}, port);

	while server.process() {
		// spin
	}

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}
