use crate::mmu::PageTable;
use crate::mmu::PhysAddr;

use crate::KERNEL_ADDRESS_SPACE;
use core::arch::asm;

pub fn enable_mmu() {
    KERNEL_ADDRESS_SPACE.read().make_active();
    // Assume all the other flags are fine. Maybe.
}

pub unsafe fn switch_to_page_table(phys_addr: PhysAddr) {
    asm!("mov cr3, {phys}", phys = in (reg) (phys_addr.0));
}

pub unsafe fn invalidate_tlb_for_range(start: usize, size: usize) {
    for page in (start..(start + size)).step_by(0x1000) {
        asm!("invlpg [{page}]", page = in (reg) (page));
    }
}

pub unsafe fn read_cr3() -> PhysAddr {
    let cr3: usize;
    asm!("mov {phys}, cr3", phys = out(reg)(cr3));
    PhysAddr(cr3)
}

pub unsafe fn get_current_page_table() -> &'static PageTable {
    let current_pages_phys = read_cr3();
    let current_pages_virt: *const PageTable =
        crate::mmu::phys_to_virt(current_pages_phys) as *const PageTable;
    current_pages_virt.as_ref().unwrap()
}
