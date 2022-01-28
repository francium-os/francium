use crate::phys_allocator;
use crate::mmu::{PageTable, PagePermission, MapType};
use crate::PhysAddr;
use spin::RwLock;
use smallvec::SmallVec;
use crate::arch::aarch64;

lazy_static! {
	pub static ref KERNEL_ADDRESS_SPACE: RwLock<AddressSpace> = RwLock::new(AddressSpace::new(PageTable::new()));
}

#[derive(Debug)]
struct Block {
	address: usize,
	size: usize,
	permissions: PagePermission
}

pub struct AddressSpace {
	pub page_table: &'static mut PageTable,
	page_table_phys: PhysAddr,
	regions: SmallVec<[Block; 4]>
}

impl core::fmt::Debug for AddressSpace {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("AddressSpace").finish()
	}
}

impl AddressSpace {
	pub fn new(template_page_table: PageTable) -> AddressSpace {
		// Crimes activated
		// This will only really work if pagetable is exactly a page big... and we never free it.
		unsafe {
			let phys_page = match phys_allocator::alloc() {
				Some(x) => x,
				None => panic!("Out of physical memory!")
			};

			let page_table_ptr = crate::mmu::phys_to_virt(phys_page) as *mut PageTable;
			page_table_ptr.copy_from(&template_page_table as *const PageTable, 1);

			let page_table = match page_table_ptr.as_mut() {
				Some(x) => x,
				None => panic!("Somehow phys_to_virt returned null?")
			};

			AddressSpace {
				page_table: page_table,
				page_table_phys: phys_page,
				regions: SmallVec::new()
			}
		}
	}

	pub fn alias(&mut self, start_phys: PhysAddr, start_addr: usize, size: usize, perm: PagePermission) {
		for addr in (start_addr..(start_addr+size)).step_by(0x1000) {
			let page = PhysAddr(start_phys.0 + (addr - start_addr));
			self.page_table.map_4k(page, addr, perm, MapType::NormalCachable);
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
				self.page_table.map_4k(page, addr, perm, MapType::NormalCachable);
			}
		}

		self.regions.push(Block{
			address: start_addr,
			size: size,
			permissions: perm
		})
	}

	pub fn expand(&mut self, start_addr: usize, new_size: usize) {

		for r in &mut self.regions {
			if r.address == start_addr {
				// etc
				// TODO: page coalescing, etc.
				// For now, dumb ass 4k pages.

				if r.size > new_size {
					// Wtf are you doing trying to shrink?
					panic!("Stop it! expand called with smaller size");
				}

				unsafe {
					for offset in (r.size .. new_size).step_by(0x1000) {
						let page = phys_allocator::alloc().unwrap();
						self.page_table.map_4k(page, r.address+offset, r.permissions, MapType::NormalCachable);
					}
				}

				r.size = new_size;
				return
			}
		}
		panic!("Wtf?");
	}

	pub fn make_active(&self) {
		unsafe {
			aarch64::set_ttbr0_el1(self.page_table_phys);
			aarch64::set_ttbr1_el1(self.page_table_phys);
		}
	}
}