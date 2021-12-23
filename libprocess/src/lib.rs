#![no_std]
#![feature(lang_items)]
#![feature(panic_info_message)]

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Handle(u32);
//const INVALID_HANDLE: Handle = Handle(0xffffffff);

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct ResultCode(u32);
const RESULT_OK: ResultCode = ResultCode(0);

#[macro_use]
pub mod print;
pub mod os_error;
pub mod syscalls;
mod lang_items;