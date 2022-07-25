pub mod msr;
pub mod context;
pub mod mmu;
pub mod cache;
pub mod gdt;
mod interrupt_handlers;
pub mod idt;
pub mod syscall;
pub mod svc_wrappers;

use core::arch::global_asm;
global_asm!(include_str!("asm/stack.s"));
global_asm!(include_str!("asm/context.s"));
global_asm!(include_str!("asm/scheduler.s"));