#![no_std]
#![feature(lang_items)]
#![feature(panic_info_message)]

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Handle(u32);
const INVALID_HANDLE: Handle = Handle(0xffffffff);

#[macro_use]
pub mod print;
pub mod syscalls;
mod lang_items;