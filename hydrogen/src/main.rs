#![no_std]
#![feature(lang_items)]

use process::syscalls;

fn main() {
	syscalls::connect_to_port("sm");
	syscalls::exit_process();
}
