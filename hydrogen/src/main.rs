#![no_std]
#![feature(lang_items)]

use process::syscalls;

fn main() {
	syscalls::print("process two this is also long");
	loop {}
}
