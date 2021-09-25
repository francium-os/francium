#![no_std]

extern crate process;
use process::syscalls;

fn main() {
	loop {
		syscalls::print("process one");
	}
}
