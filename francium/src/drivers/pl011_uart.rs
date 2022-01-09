use crate::constants::*;
use crate::PhysAddr;

pub struct Pl011Uart {
	base_address: PhysAddr,
	baud: u32
}

impl Pl011Uart {
	pub fn new(base_address: PhysAddr, baud: u32) -> Pl011Uart {
		// todo: set up baud, etc

		Pl011Uart {
			base_address: base_address,
			baud: baud
		}
	}

	pub fn write_string(&mut self, a: &str) {
		let uart_base: *mut u8 = (PHYSMAP_BASE + self.base_address.0) as *mut u8;
		for c in a.chars() {
			unsafe {
				uart_base.write_volatile(c as u8);
			}
		}
	}

	pub fn write_bytes(&mut self, a: &[u8]) {
		let uart_base: *mut u8 = (PHYSMAP_BASE + self.base_address.0) as *mut u8;
		for c in a {
			unsafe {
				uart_base.write_volatile(*c);
			}
		}
	}
}