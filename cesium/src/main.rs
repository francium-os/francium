#![no_std]

use process::println;
use process::syscalls;

fn main() {
	println!("Creating sm port...");
	let port = syscalls::create_port("sm").unwrap();
	println!("Created sm port: {:?}.", port);
	//syscalls::ipc_receive(port).unwrap();

	syscalls::close_handle(port).unwrap();
	syscalls::exit_process();
}