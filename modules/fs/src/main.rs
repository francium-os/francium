#![no_std]
#![feature(default_alloc_error_handler)]

use process::println;
use process::syscalls;
use process::Handle;
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::fs;

type FSServer = ServerImpl<fs::FSServerStruct>;

fn main() {
	println!("Hello from fs!");

	let port = syscalls::create_port("fs").unwrap();
	let mut server = FSServer::new(port);

	while server.process() {
		// spin
	}

	syscalls::close_handle(port).unwrap();
	println!("FS exiting!");

	syscalls::exit_process();
}
