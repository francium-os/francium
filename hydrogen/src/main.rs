#![no_std]

use process::println;
use process::syscalls;

fn main() {
	println!("Connecting to sm...");
	let port = syscalls::connect_to_port("sm").unwrap();
	println!("Connected to sm port! {:?}", port);
	//syscalls::ipc_request(port).unwrap();

	syscalls::close_handle(port).unwrap();
	syscalls::exit_process();
}