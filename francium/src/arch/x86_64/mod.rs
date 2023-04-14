pub use francium_x86::*;
pub mod gdt;
pub mod idt;
pub mod info;
mod interrupt_handlers;
pub mod mmu;
mod svc_wrappers;
pub mod syscall;

use core::arch::global_asm;
global_asm!(include_str!("asm/stack.s"));
global_asm!(include_str!("asm/context.s"));
global_asm!(include_str!("asm/scheduler.s"));


pub fn setup_per_cpu() {
	unsafe {
		let per_cpu_base = crate::per_cpu::get_base();
		crate::per_cpu::get().per_cpu_ptr = per_cpu_base;
		msr::write_kernel_gs_base(per_cpu_base);
	}
}