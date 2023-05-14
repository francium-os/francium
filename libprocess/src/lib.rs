#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(thread_local)]

extern crate alloc;

//pub mod print;
pub mod ipc_server;
pub mod syscalls;
//pub mod allocator;
pub mod ipc;
pub mod os_error;

//pub use common::os_error;
pub use common::{Handle, INVALID_HANDLE};
