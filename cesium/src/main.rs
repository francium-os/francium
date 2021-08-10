#![no_std]
#![feature(lang_items)]

pub mod bleh;


extern "C" {
	pub fn syscall_print(s: *const u8);
}

fn main() {
	unsafe {
		syscall_print(b"testing!\n" as *const u8);
	}
	loop {}
}
