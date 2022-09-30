#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(thread_local)]

use process::println;
use process::syscalls;
use process::ipc;

fn main() {
	println!("Hello from test!");

	let fs_handle = ipc::sm::get_service_handle(syscalls::make_tag("fs")).unwrap();
	println!("fs handle: {:?}", fs_handle);
	println!("[C] Client done!");
	syscalls::exit_process();
}