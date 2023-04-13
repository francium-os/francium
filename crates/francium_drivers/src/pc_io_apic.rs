// I can't use tock_registers here. Because Intel is very smart. Thanks, Intel.
use crate::InterruptDistributor;

struct IOApic {
	base: *mut u32
}

impl IOApic {
	fn new(base_address_virt: usize) -> IOApic {
		IOApic  {
			base: base_address_virt as *mut u32
		}
	}

	unsafe fn read_register(&mut self, index: u32) -> u32 {
		self.base.write_volatile(index);
		self.base.add(4).read_volatile() // offset 0x10
	}

	unsafe fn write_register(&mut self, index: u32, value: u32) {
		self.base.write_volatile(index);
		self.base.add(4).write_volatile(value) // offset 0x10
	}
}

/*impl InterruptDistributor for IOApic {
	
}*/