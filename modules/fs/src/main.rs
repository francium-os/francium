#![no_std]
#![feature(default_alloc_error_handler)]

use process::println;
use process::syscalls;

use process::ipc_server::{ServerImpl, IPCServer};

struct FSCallback {
}

impl IPCServer for FSCallback {
	fn handle() {
		println!("FS message!");
	}
}

type FSServer = ServerImpl<FSCallback>;

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
