#![no_std]

use process::print;
use process::syscalls;

fn main() {
	print!("Connecting to sm...");
	let port = syscalls::connect_to_port("sm").unwrap();
	print!("Connected to sm port!");
	syscalls::close_handle(port);
	syscalls::exit_process();
}