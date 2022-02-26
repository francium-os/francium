use spin::Mutex;
use crate::constants::*;
use crate::PhysAddr;

pub const PHYS_MEM_BASE: usize = 0;

pub struct COMPort {}
impl COMPort {
	pub fn write_string(&mut self, a: &str) {
		// todo
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

use core::arch::global_asm;
global_asm!(include_str!("../arch/x86_64/asm/stub.s"));