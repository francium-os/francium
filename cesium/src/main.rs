#![no_std]

use process::println;
use process::syscalls;

fn main() {
	println!("Creating sm port...");
	let port = syscalls::create_port("sm").unwrap();
	syscalls::close_handle(port);
	syscalls::exit_process();
}