#![no_std]

use process::println;
use process::syscalls;

fn main() {
	println!("Connecting to sm...");
	let port = syscalls::connect_to_port("sm").unwrap();
	syscalls::close_handle(port);
	syscalls::exit_process();
}