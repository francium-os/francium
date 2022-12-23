use crate::phys_allocator;
use crate::constants::PHYSMAP_BASE;

use francium_common::types::PhysAddr;
pub use francium_common::types::{MapType, PagePermission};
pub use francium_mmu::*;

pub struct FranciumPhysAccess {}
impl francium_mmu::PhysAccess for FranciumPhysAccess {
	fn phys_to_virt(phys: PhysAddr) -> usize {
		phys.0 + PHYSMAP_BASE
	}
}


pub struct FranciumPhysAlloc {}
impl francium_mmu::PhysAlloc for FranciumPhysAlloc {
	fn alloc() -> Option<PhysAddr> {
		unsafe { phys_allocator::alloc() }
	}
}

#[cfg(target_arch = "x86_64")]
use francium_x86::page_table::X86_64Specific as ArchSpecific;

#[cfg(target_arch = "aarch64")]
use francium_aarch64::page_table::AArch64Specific as ArchSpecific;

pub type PageTable = francium_mmu::PageTable<ArchSpecific, FranciumPhysAlloc, FranciumPhysAccess>;

pub fn phys_to_virt(phys: PhysAddr) -> usize {
    FranciumPhysAccess::phys_to_virt(phys)
}

use crate::arch::mmu;
pub fn enable_mmu() {
    mmu::enable_mmu();
}