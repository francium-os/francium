use crate::os_error::{OSError, ResultCode, RESULT_OK};
use common::system_info::*;
use common::{Handle, INVALID_HANDLE};
use common::{MapType, PagePermission};
use core::cmp::min;

pub fn print(s: &str) {
    todo!();
}

pub fn make_tag(s: &str) -> u64 {
    let tag_bytes = s.as_bytes();
    let length = min(8, tag_bytes.len());
    let mut tag_bytes_padded: [u8; 8] = [0; 8];
    tag_bytes_padded[0..length].copy_from_slice(tag_bytes);

    u64::from_be_bytes(tag_bytes_padded)
}

pub fn create_port(s: &str) -> Result<Handle, OSError> {
    todo!();
}

pub fn connect_to_named_port(s: &str) -> Result<Handle, OSError> {
    todo!();
}

pub fn connect_to_port_handle(h: Handle) -> Result<Handle, OSError> {
    todo!();
}

pub fn close_handle(h: Handle) -> Result<(), OSError> {
    todo!();
}

pub fn exit_process() -> ! {
    todo!();
}

pub fn ipc_request(session_handle: Handle, ipc_buffer: &mut [u8; 128]) -> Result<(), OSError> {
    todo!();
}

pub fn ipc_reply(session_handle: Handle, ipc_buffer: &mut [u8; 128]) -> Result<(), OSError> {
    todo!();
}

pub fn ipc_receive(sessions: &[Handle], ipc_buffer: &mut [u8; 128]) -> Result<usize, OSError> {
    todo!();
}

pub fn ipc_accept(session_handle: Handle) -> Result<Handle, OSError> {
    todo!();
}

pub fn get_process_id() -> u64 {
    todo!();
}

pub fn map_memory(
    address: usize,
    length: usize,
    permission: PagePermission,
) -> Result<usize, OSError> {
    todo!();
}

pub fn sleep_ns(ns: u64) {
    todo!();
}

pub use common::constants::{GET_FS, SET_FS};

pub fn bodge(key: u32, addr: usize) -> usize {
    todo!();
}

pub fn get_thread_id() -> u64 {
    todo!();
}

pub fn map_device_memory(
    phys_addr: usize,
    virt_addr: usize,
    length: usize,
    ty: MapType,
    permission: PagePermission,
) -> Result<usize, OSError> {
    todo!();
}

pub fn get_system_info(ty: SystemInfoType, index: usize) -> Result<SystemInfo, OSError> {
    todo!();
}

pub fn get_system_tick() -> u64 {
    todo!();
}

pub fn query_physical_address(virt: usize) -> Result<usize, OSError> {
    todo!();
}

pub fn create_event() -> Result<Handle, OSError> {
    todo!();
}

pub fn bind_interrupt(handle: Handle, index: usize) -> Result<(), OSError> {
    todo!();
}

pub fn unbind_interrupt(handle: Handle, index: usize) -> Result<(), OSError> {
    todo!();
}

pub fn wait_one(handle: Handle) -> Result<(), OSError> {
    todo!();
}

pub fn signal_event(handle: Handle) -> Result<(), OSError> {
    todo!();
}

pub fn clear_event(handle: Handle) -> Result<(), OSError> {
    todo!();
}

pub fn wait_many(handles: &[Handle]) -> Result<usize, OSError> {
    todo!();
}

pub fn create_session() -> Result<(Handle, Handle), OSError> {
    todo!();
}
