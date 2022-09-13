use core::arch::asm;

pub unsafe fn read_tpidr_el0() -> usize {
	let mut value: usize;
	asm!("mrs {value}, tpidr_el0", value = out (reg) (value));
	value
}

pub unsafe fn write_tpidr_el0(tpidr_el0: usize) {
	asm!("msr tpidr_el0, {value}", value = in (reg) (tpidr_el0));
}