use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::mmu::PageTable;
use francium_common::types::PhysAddr;
use core::arch::asm;

unsafe fn get_ttbr0_el1() -> PhysAddr {
    let mut value: usize;
    asm!("mrs {ttbr0_el1}, ttbr0_el1", ttbr0_el1 = out(reg) value);
    PhysAddr(value)
}

pub unsafe fn set_ttbr0_el1(value_phys: PhysAddr) {
    let value = value_phys.0;
    asm!("msr ttbr0_el1, {ttbr0_el1}
		  tlbi vmalle1
		  dsb ish
		  isb", ttbr0_el1 = in(reg) value);
}

unsafe fn get_ttbr1_el1() -> PhysAddr {
    let mut value: usize;
    asm!("mrs {ttbr1_el1}, ttbr1_el1", ttbr1_el1 = out(reg) value);
    PhysAddr(value)
}

pub unsafe fn set_ttbr1_el1(value_phys: PhysAddr) {
    let value = value_phys.0;
    asm!("msr ttbr1_el1, {ttbr1_el1}
		  tlbi vmalle1
		  dsb ish
		  isb", ttbr1_el1 = in(reg) value);
}

pub unsafe fn set_sctlr_el1(value: usize) {
    asm!("msr sctlr_el1, {sctlr_el1}", sctlr_el1 = in(reg) value);
}

pub unsafe fn get_tcr_el1() -> usize {
    let mut value: usize;
    asm!("msr {tcr_el1}, tcr_el1", tcr_el1 = out(reg) value);
    value
}

pub unsafe fn set_tcr_el1(value: usize) {
    asm!("msr tcr_el1, {tcr_el1}", tcr_el1 = in(reg) value);
}

pub fn enable_mmu() {
    KERNEL_ADDRESS_SPACE.read().make_active();

    unsafe {
        // enable caches + mmu
        // enable sp alignment?

        const SCTLR_LSMAOE: usize = 1 << 29;
        const SCTLR_NTLSMD: usize = 1 << 28;
        const SCTLR_TSCXT: usize = 1 << 20;
        //const SCTLR_ITD = 1<<7;

        const SCTLR_I: usize = 1 << 12;
        const SCTLR_SPAN: usize = 1 << 3;
        const SCTLR_C: usize = 1 << 2;
        const SCTLR_M: usize = 1 << 0;

        const TCR_IPS_48_BIT: usize = 0b101 << 32;
        const TCR_TG1_GRANULE_4K: usize = 0 << 30;
        const TCR_TG0_GRANULE_4K: usize = 0 << 14;

        const TCR_T0SZ_48_BIT: usize = 16;
        const TCR_T1SZ_48_BIT: usize = 16 << 16;

        let tcr = TCR_IPS_48_BIT
            | TCR_TG0_GRANULE_4K
            | TCR_TG1_GRANULE_4K
            | TCR_T0SZ_48_BIT
            | TCR_T1SZ_48_BIT;
        set_tcr_el1(tcr);

        // RES1 bits
        let mut sctlr = SCTLR_LSMAOE | SCTLR_NTLSMD | SCTLR_TSCXT;

        // icache, dcache, sp alignment, mmu enable
        sctlr |= SCTLR_I | SCTLR_SPAN | SCTLR_C | SCTLR_M;
        set_sctlr_el1(sctlr);
    }
}

// > &'static
pub unsafe fn get_current_page_table() -> &'static PageTable {
    let ttbr0 = get_ttbr0_el1();
    let ttbr1 = get_ttbr1_el1();
    assert!(ttbr0.0 == ttbr1.0);

    let current_pages_virt: *const PageTable = crate::mmu::phys_to_virt(ttbr1) as *const PageTable;
    current_pages_virt.as_ref().unwrap()
}

pub unsafe fn switch_to_page_table(phys_addr: PhysAddr) {
    set_ttbr0_el1(phys_addr);
    set_ttbr1_el1(phys_addr);
}

pub unsafe fn invalidate_tlb_for_range(_start: usize, _size: usize) {
    // TODO: actual TLB management
    asm!("tlbi vmalle1");
}
