use core::arch::asm;

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