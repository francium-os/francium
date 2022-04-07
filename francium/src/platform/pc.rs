use core::arch::asm;

use spin::Mutex;
use crate::constants::*;
use crate::mmu::PhysAddr;

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // 2gb?? for now

pub struct COMPort {}
impl COMPort {
	pub fn write_byte(&mut self, byte: u8) {
		unsafe {
			asm!("out dx, al", in("dx") 0x3f8, in("al") byte);
		}
	}

	pub fn write_string(&mut self, a: &str) {
		for c in a.chars() {
			self.write_byte(c as u8);
		}
	}

	pub fn write_bytes(&mut self, a: &[u8]) {
		for c in a {
			self.write_byte(*c);
		}
	}
}

lazy_static! {
	pub static ref DEFAULT_UART: Mutex<COMPort> = Mutex::new(COMPort{});
}

pub fn platform_specific_init() {
	// Nothing, for now
}

pub fn scheduler_pre_init() {
	unimplemented!();
}

pub fn scheduler_post_init() {
	unimplemented!();
}
