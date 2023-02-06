use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::mmu::PageTable;
use core::arch::asm;
use francium_common::types::PhysAddr;

use aarch64_cpu::{asm::barrier, registers::*};
use tock_registers::interfaces::{Readable, Writeable};

pub fn enable_mmu() {
    KERNEL_ADDRESS_SPACE.read().make_active();

    TCR_EL1.write(
        TCR_EL1::IPS::Bits_48
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::TG1::KiB_4
            + TCR_EL1::T1SZ.val(16)
            + TCR_EL1::T0SZ.val(16),
    );

    barrier::isb(barrier::SY);

    SCTLR_EL1.write(
        SCTLR_EL1::SA0::Enable
            + SCTLR_EL1::SA::Enable
            + SCTLR_EL1::M::Enable
            + SCTLR_EL1::C::Cacheable
            + SCTLR_EL1::I::Cacheable,
    );

    barrier::isb(barrier::SY);
}

// > &'static
pub unsafe fn get_current_page_table() -> &'static PageTable {
    let ttbr0 = TTBR0_EL1.get();
    let ttbr1 = TTBR1_EL1.get();
    assert!(ttbr0 == ttbr1);

    let current_pages_virt: *const PageTable =
        crate::mmu::phys_to_virt(PhysAddr(ttbr1 as usize)) as *const PageTable;
    current_pages_virt.as_ref().unwrap()
}

pub unsafe fn switch_to_page_table(phys_addr: PhysAddr) {
    TTBR0_EL1.set(phys_addr.0 as u64);
    TTBR1_EL1.set(phys_addr.0 as u64);

    asm!("tlbi vmalle1");
    barrier::dsb(barrier::ISH);
    barrier::isb(barrier::SY);
}

pub unsafe fn invalidate_tlb_for_range(_start: usize, _size: usize) {
    // TODO: actual TLB management
    asm!("tlbi vmalle1");
}
