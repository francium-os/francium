#![no_std]

use process::println;
use process::syscalls;

fn main() {
	println!("[C] Connecting to sm...");
	let port = syscalls::connect_to_port("sm").unwrap();
	println!("[C] Connected to sm port! {:?}", port);
	println!("[C] Doing an IPC request");
	syscalls::ipc_request(port).unwrap();
	println!("[C] Done with request!");

	syscalls::close_handle(port).unwrap();
	println!("[C] Client done!");
	syscalls::exit_process();
}