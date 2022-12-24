use francium_common::types::PhysAddr;
use core::arch::asm;

pub unsafe fn get_ttbr0_el1() -> PhysAddr {
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

pub unsafe fn get_ttbr1_el1() -> PhysAddr {
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

pub unsafe fn get_ttbr0_el2() -> PhysAddr {
    let mut value: usize;
    asm!("mrs {ttbr0_el2}, ttbr0_el2", ttbr0_el2 = out(reg) value);
    PhysAddr(value)
}

pub unsafe fn set_ttbr0_el2(value_phys: PhysAddr) {
    let value = value_phys.0;
    asm!("msr ttbr0_el2, {ttbr0_el2}
          tlbi vmalle1
          dsb ish
          isb", ttbr0_el2 = in(reg) value);
}

pub unsafe fn set_sctlr_el1(value: usize) {
    asm!("msr sctlr_el1, {sctlr_el1}", sctlr_el1 = in(reg) value);
}

pub unsafe fn get_tcr_el1() -> usize {
    let mut value: usize;
    asm!("mrs {tcr_el1}, tcr_el1", tcr_el1 = out(reg) value);
    value
}

pub unsafe fn set_tcr_el1(value: usize) {
    asm!("msr tcr_el1, {tcr_el1}", tcr_el1 = in(reg) value);
}
