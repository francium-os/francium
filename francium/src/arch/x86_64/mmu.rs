use crate::mmu::PhysAddr;
use crate::KERNEL_ADDRESS_SPACE;
use core::arch::asm;

pub fn enable_mmu() {
	KERNEL_ADDRESS_SPACE.read().make_active();
	// Assume all the other flags are fine. Maybe.
}

pub fn switch_to_page_table(phys_addr: PhysAddr) {
	unsafe {
		asm!("mov cr3, {phys}", phys = in (reg) (phys_addr.0));
	}
}