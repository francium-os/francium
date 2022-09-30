#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use process::println;
use process::syscalls;
use process::{Handle, INVALID_HANDLE};
use process::os_error::{OSError, OSResult, Module, Error};
use process::ipc_server::ServerImpl;
use process::ipc::sm::SMServer;

struct SMServerStruct {}

impl SMServer for SMServerStruct {
	fn get_service_handle(&self, tag: u64) -> OSResult<Handle> {
		println!("Got tag: {:x}", tag);
		Ok(INVALID_HANDLE)
	}
}

type SMServerImpl = ServerImpl<Box<dyn SMServer>>;

fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let mut server = SMServerImpl::new(Box::new(SMServerStruct{}), port);

	while server.process() {
		// spin
	}

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}
