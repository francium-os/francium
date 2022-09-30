#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use process::println;
use process::syscalls;
use process::Handle;
use process::os_error::{OSError, OSResult, Module, Error};
use process::ipc_server::ServerImpl;
use process::ipc::fs::FSServer;

struct FSServerStruct{}

impl FSServer for FSServerStruct {
	fn test(&self) -> OSResult<Handle> {
		Err(OSError { module: Module::FS, err: Error::NotImplemented })
	}
}

type FSServerImpl = ServerImpl<Box<dyn FSServer>>;

fn main() {
	println!("Hello from fs!");

	let port = syscalls::create_port("fs").unwrap();
	let mut server = FSServerImpl::new(Box::new(FSServerStruct{}), port);

	while server.process() {
		// spin
	}

	syscalls::close_handle(port).unwrap();
	println!("FS exiting!");

	syscalls::exit_process();
}
