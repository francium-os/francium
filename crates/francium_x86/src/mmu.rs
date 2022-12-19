use francium_common::types::PhysAddr;
use core::arch::asm;

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