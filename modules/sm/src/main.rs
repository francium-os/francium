#![no_std]
#![feature(default_alloc_error_handler)]

use process::println;
use process::syscalls;
use process::Handle;

use process::ipc_server::{ServerImpl, IPCServer};

struct SMCallback {
}

impl IPCServer for SMCallback {
	fn handle(h: Handle) {
		println!("SM message!");
		syscalls::ipc_reply(h).unwrap();
	}
}

type SMServer = ServerImpl<SMCallback>;

fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let mut server = SMServer::new(port);

	while server.process() {
		// spin
	}

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}
