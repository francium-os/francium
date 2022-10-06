#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(thread_local)]

use process::println;
use process::syscalls;
use process::ipc;

fn main() {
	println!("Hello from test!");

	println!("FS IPC");
	ipc::fs::stop();
	println!("SM IPC");
	ipc::sm::stop();

	println!("Done");

	syscalls::exit_process();
}