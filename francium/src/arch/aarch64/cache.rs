use core::arch::asm;

pub unsafe fn clear_cache_for_address(addr: usize) {
	asm!("dc cvau, {addr}
		  ic ivau, {addr}", addr = in (reg) (addr));
}