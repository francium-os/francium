use core::arch::asm;

pub unsafe fn clear_cache_for_address(addr: usize) {
	asm!("clflush [{addr}]", addr = in (reg) (addr));
}