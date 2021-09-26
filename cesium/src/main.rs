#![no_std]

extern crate process;
use process::syscalls;

fn main() {
	syscalls::create_port("sm");
	syscalls::exit_process();
}
