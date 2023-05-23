#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(thread_local)]

extern crate alloc;

//pub mod print;
pub mod ipc_server;

#[cfg(target_os = "francium")]
#[path = "syscalls_native.rs"]
pub mod syscalls;
#[cfg(not(target_os = "francium"))]
#[path = "syscalls_emulated.rs"]
pub mod syscalls;

//pub mod allocator;
pub mod ipc;
pub mod os_error;

//pub use common::os_error;
pub use common::{Handle, INVALID_HANDLE};
