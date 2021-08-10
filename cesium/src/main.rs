#![no_std]
#![feature(lang_items)]

pub mod bleh;
pub mod syscalls;

fn main() {
	syscalls::print("gaming");
	loop {}
}
