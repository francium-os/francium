#![no_std]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(thread_local)]

extern crate alloc;

#[macro_use]
pub mod print;
pub mod syscalls;
mod lang_items;
pub mod ipc_server;
pub mod allocator;
pub mod ipc;

pub use common::os_error;
pub use common::{Handle, INVALID_HANDLE};