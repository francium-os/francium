pub mod arch_registers;
pub mod arch_timer;
pub mod cache;
pub mod context;
pub mod gicv2;
pub mod interrupt;
pub mod mmu;
pub mod svc_wrappers;

pub use interrupt::enable_interrupts;
pub use mmu::enable_mmu;
pub use mmu::set_ttbr0_el1;
pub use mmu::set_ttbr1_el1;

use core::arch::global_asm;
global_asm!(include_str!("asm/context.s"));
global_asm!(include_str!("asm/interrupt.s"));
global_asm!(include_str!("asm/kernel_entry.s"));
global_asm!(include_str!("asm/memory.s"));
global_asm!(include_str!("asm/misc.s"));
global_asm!(include_str!("asm/scheduler.s"));
