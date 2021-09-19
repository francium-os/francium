#![no_std]

extern crate process;
use process::syscalls;

fn main() {
	syscalls::print("process one this is long");
	loop {}
}
