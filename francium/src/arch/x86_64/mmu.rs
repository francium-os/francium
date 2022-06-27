use crate::mmu::PhysAddr;
use crate::mmu::PageTable;

use crate::KERNEL_ADDRESS_SPACE;
use core::arch::asm;

extern "C" {
	fn read_cr3() -> PhysAddr;
}

pub fn enable_mmu() {
	KERNEL_ADDRESS_SPACE.read().make_active();
	// Assume all the other flags are fine. Maybe.
}

pub fn switch_to_page_table(phys_addr: PhysAddr) {
	unsafe {
		asm!("mov cr3, {phys}", phys = in (reg) (phys_addr.0));
	}
}

pub unsafe fn get_current_page_table() -> &'static PageTable {
	let current_pages_phys = read_cr3();
	let current_pages_virt: *const PageTable = crate::mmu::phys_to_virt(current_pages_phys) as *const PageTable;
    current_pages_virt.as_ref().unwrap()
}