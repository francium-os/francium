use crate::phys_allocator;
use crate::mmu::{PageTable, PagePermission};
use crate::PhysAddr;
use spin::RwLock;
use alloc::vec::Vec;

lazy_static! {
	pub static ref KERNEL_ADDRESS_SPACE: RwLock<AddressSpace> = RwLock::new(AddressSpace::new(PageTable::new()));
}

struct Block {
	address: usize,
	size: usize,
	permissions: PagePermission
}

pub struct AddressSpace {
	pub page_table: PageTable,
	regions: Vec<Block>
}

impl AddressSpace {
	pub fn new(page_table: PageTable) -> AddressSpace {
		AddressSpace {
			page_table: page_table,
			regions: Vec::new()
		}
	}

	pub fn alias(&mut self, start_phys: PhysAddr, start_addr: usize, size: usize, perm: PagePermission) {
		unsafe {
			for addr in (start_addr..(start_addr+size)).step_by(0x1000) {
				let page = PhysAddr(start_phys.0 + (addr - start_addr));
				self.page_table.map_4k(page, addr, perm);
			}
		}
		self.regions.push(Block{
			address: start_addr,
			size: size,
			permissions: perm
		})
	}

	pub fn create(&mut self, start_addr: usize, size: usize, perm: PagePermission) {
		unsafe {
			for addr in (start_addr..(start_addr+size)).step_by(0x1000) {
				let page = phys_allocator::alloc().unwrap();
				self.page_table.map_4k(page, addr, perm);
			}
		}

		self.regions.push(Block{
			address: start_addr,
			size: size,
			permissions: perm
		})
	}
}