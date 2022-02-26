#![no_std]

use process::println;
use process::syscalls;
use process::Handle;

fn main() {
	println!("[C] Hello from cesium! My process id is {:?}", syscalls::get_process_id());
	println!("[S] Creating sm port...");
	let port = syscalls::create_port("sm").unwrap();
	println!("[S] Created sm port: {:?}.", port);
	//syscalls::ipc_receive(port).unwrap();

	let handles: [Handle; 1] = [port];
	let index = syscalls::ipc_receive(&handles).unwrap();
	println!("[S] Got index? {:?}", index);
	if index == 0 {
		let new_session = syscalls::ipc_accept(port).unwrap();
		println!("[S] Got new session {:?}", new_session);

		let handles: [Handle; 2] = [port, new_session];
		let index = syscalls::ipc_receive(&handles).unwrap();
		println!("[S] Got ipc result {:?}", index);
		if index == 1 {
			syscalls::ipc_reply(new_session).unwrap();
		}
	}

	syscalls::close_handle(port).unwrap();
	println!("[S] Server done!");

	syscalls::exit_process();
}
