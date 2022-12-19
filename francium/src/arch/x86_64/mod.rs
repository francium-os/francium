pub use francium_x86::*;
pub mod syscall;
pub mod info;
mod svc_wrappers;
mod interrupt_handlers;
pub mod gdt;
pub mod idt;
pub mod mmu;

use core::arch::global_asm;
global_asm!(include_str!("asm/stack.s"));
global_asm!(include_str!("asm/context.s"));
global_asm!(include_str!("asm/scheduler.s"));
