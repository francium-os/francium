use core::arch::asm;
use crate::arch::msr;
use spin::Mutex;

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // 2gb?? for now

pub struct COMPort {
	port_base: u16
}

impl COMPort {
	pub fn new(port_base: u16) -> COMPort {
		unsafe {
			asm!("out dx, al", in("dx") port_base + 3, in("al") 3 as u8);
		}

		COMPort{port_base}
	}

	pub fn write_byte(&mut self, byte: u8) {
		unsafe {
			let line_status_reg = self.port_base+5;

			let mut line_status: u8;
			asm!("in al, dx", out("al") line_status, in("dx") line_status_reg);
			while (line_status & (1<<5)) != (1<<5) {
				asm!("in al, dx", out("al") line_status, in("dx") line_status_reg);
			}

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

unsafe fn turn_on_floating_point() {
	core::arch::asm!("
		mov rax, cr0
		and ax, 0xFFFB
		or ax, 0x2
		mov cr0, rax
		mov rax, cr4
		or ax, 3 << 9
		mov cr4, rax
	");
}

lazy_static! {
	pub static ref DEFAULT_UART: Mutex<COMPort> = Mutex::new(COMPort::new(0x3f8));
}

pub fn platform_specific_init() {
}

pub fn scheduler_pre_init() {
	//unimplemented!();
}

pub fn scheduler_post_init() {
	unsafe { turn_on_floating_point(); }

	// XXX give this a constant
	// Enable XN
	unsafe { msr::write_efer(msr::read_efer() | (1<<11)); }
}
