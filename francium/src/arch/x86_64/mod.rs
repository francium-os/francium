pub mod msr;
pub mod context;
pub mod mmu;
pub mod cache;
pub mod gdt;
mod interrupt_handlers;
pub mod idt;
pub mod syscall;

use core::arch::global_asm;
global_asm!(include_str!("asm/stack.s"));
global_asm!(include_str!("asm/context.s"));
