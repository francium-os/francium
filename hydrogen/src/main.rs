#![no_std]
#![feature(lang_items)]

use process::syscalls;

fn main() {
	loop {
		syscalls::print("process two");
	}
}
