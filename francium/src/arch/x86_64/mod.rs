pub use francium_x86::*;
pub mod gdt;
pub mod idt;
pub mod info;
mod interrupt_handlers;
pub mod mmu;
pub mod per_cpu;
mod svc_wrappers;
pub mod syscall;

use core::arch::global_asm;
global_asm!(include_str!("asm/stack.s"));
global_asm!(include_str!("asm/context.s"));
global_asm!(include_str!("asm/scheduler.s"));
global_asm!(include_str!("asm/trampoline.s"), options(att_syntax));

pub use per_cpu::get_per_cpu_base;
pub use per_cpu::setup_per_cpu;