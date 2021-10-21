#![no_std]

use process::print;
use process::syscalls;

fn main() {
	print!("Creating sm port...");
	let port = syscalls::create_port("sm").unwrap();
	print!("Created sm port.");
	syscalls::close_handle(port);
	syscalls::exit_process();
}