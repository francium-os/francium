// I can't use tock_registers here. Because Intel is very smart. Thanks, Intel.
use crate::InterruptDistributor;

const IOAPIC_REDIR_TABLE: u32 = 0x10;

pub struct IoApic {
	base: usize
}

impl IoApic {
	pub fn new(base_address_virt: usize) -> IoApic {
		IoApic  {
			base: base_address_virt
		}
	}

	unsafe fn read_register(&mut self, index: u32) -> u32 {
		(self.base as *mut u32).write_volatile(index);
		(self.base as *mut u32).add(4).read_volatile() // offset 0x10
	}

	unsafe fn write_register(&mut self, index: u32, value: u32) {
		(self.base as *mut u32).write_volatile(index);
		(self.base as *mut u32).add(4).write_volatile(value) // offset 0x10
	}
}

impl InterruptDistributor for IoApic {
	fn init(&mut self) {}
	
	fn enable_interrupt(&mut self, n: u32) {
		unsafe {
			let idx = IOAPIC_REDIR_TABLE + 2*n;
			let lower = self.read_register(idx);
			//let upper = self.read_register(idx + 1);

			let new_value = (lower & !0x100ff) | (0x20 + n);
			self.write_register(idx, new_value);
		}
	}

	fn disable_interrupt(&mut self, n: u32) {
		unsafe {
			let idx = IOAPIC_REDIR_TABLE + 2*n;

			let lower = self.read_register(idx);
			//let upper = self.read_register(idx + 1);
			let new_value = lower | 0x10000;
			self.write_register(idx, new_value);
		}
	}
}