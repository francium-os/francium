#![no_std]
#![feature(naked_functions)]

extern crate alloc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod print;

pub mod align;
pub mod constants;

pub use francium_drivers as drivers;

pub mod panic;
pub mod platform;

pub mod bump_allocator;
pub mod handle;
pub mod handle_table;
pub mod mmu;
pub mod phys_allocator;

pub mod arch;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod svc;
pub mod timer;
pub mod waitable;

pub mod init;
pub mod subscriber;
pub mod acpi;

use crate::memory::KERNEL_ADDRESS_SPACE;