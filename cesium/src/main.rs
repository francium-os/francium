#![no_std]

use process::println;
use process::syscalls;
use process::Handle;

fn main() {
	println!("Creating sm port...");
	let port = syscalls::create_port("sm").unwrap();
	println!("Created sm port: {:?}.", port);
	//syscalls::ipc_receive(port).unwrap();

	let handles: [Handle; 1] = [port];
	let index = syscalls::ipc_receive(port, &handles).unwrap();
	println!("Got index? {:?}", index);
	if index == 0 {
		let new_session = syscalls::ipc_accept(port);
		println!("Got new session {:?}", new_session);
	}

	syscalls::close_handle(port).unwrap();
	syscalls::exit_process();
}