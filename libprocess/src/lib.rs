#![no_std]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(thread_local)]

extern crate alloc;

#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct Handle(u32);
pub const INVALID_HANDLE: Handle = Handle(0xffffffff);

#[macro_use]
pub mod print;
pub mod syscalls;
mod lang_items;
pub mod ipc_server;
pub mod allocator;
pub mod ipc;

pub use common::os_error;