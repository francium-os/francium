#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(thread_local)]

use process::println;
use process::syscalls;
use process::ipc_client;

#[thread_local]
pub static mut APC_BUFFER: [u8; 8] = [0xff; 8];

fn main() {
	println!("Hello from test!");
	unsafe { println!("thinky {:?} {:?}", APC_BUFFER[0], APC_BUFFER[7]); }

	let port = syscalls::connect_to_port("sm").unwrap();
	ipc_client::try_make_request(port);

	syscalls::close_handle(port).unwrap();
	println!("[C] Client done!");
	syscalls::exit_process();
}