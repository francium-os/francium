#![no_std]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(thread_local)]

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Handle(u32);
const INVALID_HANDLE: Handle = Handle(0xffffffff);

#[macro_use]
pub mod print;
pub mod syscalls;
mod lang_items;
pub mod ipc_common;
pub mod ipc_client;
pub mod ipc_server;
pub mod allocator;