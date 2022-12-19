use common::system_info::*;
use common::os_error::{OSError, ResultCode, RESULT_OK};
use common::{Handle, INVALID_HANDLE};
use core::cmp::min;

extern "C" {
    pub fn syscall_debug_output(s: *const u8, len: usize) -> ResultCode;
    pub fn syscall_create_port(tag: u64, handle_out: *mut Handle) -> ResultCode;
    pub fn syscall_connect_to_named_port(tag: u64, handle_out: *mut Handle) -> ResultCode;
    pub fn syscall_exit_process() -> !;
    pub fn syscall_close_handle(h: Handle) -> ResultCode;
    pub fn syscall_ipc_request(session_handle: Handle, ipc_buffer: *mut u8) -> ResultCode;
    pub fn syscall_ipc_reply(session_handle: Handle, ipc_buffer: *mut u8) -> ResultCode;
    pub fn syscall_ipc_receive(
        sessions: *const Handle,
        num_sessions: usize,
        ipc_buffer: *mut u8,
        index_out: *mut usize,
    ) -> ResultCode;
    pub fn syscall_ipc_accept(session_handle: Handle, handle_out: *mut Handle) -> ResultCode;
    pub fn syscall_get_process_id() -> u64;
    pub fn syscall_connect_to_port_handle(h: u32, handle_out: *mut Handle) -> ResultCode;
    pub fn syscall_map_memory(
        address: usize,
        length: usize,
        permission: u32,
        address_out: *mut usize,
    ) -> ResultCode;
    pub fn syscall_sleep_ns(ns: u64);
    pub fn syscall_bodge(key: u32, addr: usize) -> usize;
    pub fn syscall_get_thread_id() -> u64;
    pub fn syscall_map_device_memory(
        phys_addr: usize,
        virt_addr: usize,
        length: usize,
        permission: u32,
        address_out: *mut usize,
    ) -> ResultCode;

    pub fn syscall_get_system_info(
        ty: usize,
        index: usize,
        info_out: *mut usize
    ) -> ResultCode;
}

pub fn print(s: &str) {
    unsafe {
        syscall_debug_output(s.as_bytes().as_ptr(), s.len());
    }
}

pub fn make_tag(s: &str) -> u64 {
    let tag_bytes = s.as_bytes();
    let length = min(8, tag_bytes.len());
    let mut tag_bytes_padded: [u8; 8] = [0; 8];
    tag_bytes_padded[0..length].copy_from_slice(tag_bytes);

    u64::from_be_bytes(tag_bytes_padded)
}

pub fn create_port(s: &str) -> Result<Handle, OSError> {
    let mut handle_out = INVALID_HANDLE;
    unsafe {
        let res = syscall_create_port(make_tag(s), &mut handle_out);
        if res == RESULT_OK {
            Ok(handle_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn connect_to_named_port(s: &str) -> Result<Handle, OSError> {
    let mut handle_out = INVALID_HANDLE;
    unsafe {
        let res = syscall_connect_to_named_port(make_tag(s), &mut handle_out);
        if res == RESULT_OK {
            Ok(handle_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn connect_to_port_handle(h: Handle) -> Result<Handle, OSError> {
    let mut handle_out = INVALID_HANDLE;
    unsafe {
        let res = syscall_connect_to_port_handle(h.0, &mut handle_out);
        if res == RESULT_OK {
            Ok(handle_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn close_handle(h: Handle) -> Result<(), OSError> {
    unsafe {
        let res = syscall_close_handle(h);
        if res == RESULT_OK {
            Ok(())
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn exit_process() -> ! {
    unsafe {
        syscall_exit_process();
    }
}

pub fn ipc_request(session_handle: Handle, ipc_buffer: &mut [u8; 128]) -> Result<(), OSError> {
    unsafe {
        let res = syscall_ipc_request(session_handle, ipc_buffer.as_mut_ptr());
        if res == RESULT_OK {
            Ok(())
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn ipc_reply(session_handle: Handle, ipc_buffer: &mut [u8; 128]) -> Result<(), OSError> {
    unsafe {
        let res = syscall_ipc_reply(session_handle, ipc_buffer.as_mut_ptr());
        if res == RESULT_OK {
            Ok(())
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn ipc_receive(sessions: &[Handle], ipc_buffer: &mut [u8; 128]) -> Result<usize, OSError> {
    unsafe {
        let mut index_out: usize = 0;
        let res = syscall_ipc_receive(
            sessions.as_ptr(),
            sessions.len(),
            ipc_buffer.as_mut_ptr(),
            &mut index_out,
        );
        if res == RESULT_OK {
            Ok(index_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn ipc_accept(session_handle: Handle) -> Result<Handle, OSError> {
    unsafe {
        let mut handle_out: Handle = INVALID_HANDLE;
        let res = syscall_ipc_accept(session_handle, &mut handle_out);
        if res == RESULT_OK {
            Ok(handle_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn get_process_id() -> u64 {
    unsafe { syscall_get_process_id() }
}

pub fn map_memory(address: usize, length: usize, permission: u32) -> Result<usize, OSError> {
    unsafe {
        let mut address_out: usize = 0;
        let res = syscall_map_memory(address, length, permission, &mut address_out);
        if res == RESULT_OK {
            Ok(address_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn sleep_ns(ns: u64) {
    unsafe {
        syscall_sleep_ns(ns);
    }
}

pub use common::constants::{GET_FS, SET_FS};

pub fn bodge(key: u32, addr: usize) -> usize {
    unsafe { syscall_bodge(key, addr) }
}

pub fn get_thread_id() -> u64 {
    unsafe { syscall_get_process_id() }
}

pub fn map_device_memory(phys_addr: usize, virt_addr: usize, length: usize, permission: u32) -> Result<usize, OSError> {
    unsafe {
        let mut address_out: usize = 0;
        let res = syscall_map_device_memory(phys_addr, virt_addr, length, permission, &mut address_out);
        if res == RESULT_OK {
            Ok(address_out)
        } else {
            Err(OSError::from_result_code(res))
        }
    }
}

pub fn get_system_info(_ty: SystemInfoType, _index: usize) -> Result<SystemInfo, OSError> {
    unimplemented!();
    /*unsafe {
        let mut address_out: usize = 0;

        let output: SystemInfo;
        let res = syscall_get_system_info(ty as usize, index, &mut output);
        if res == RESULT_OK {
            Ok(output)
        } else {
            Err(OSError::from_result_code(res))
        }
    }*/
}


use core::arch::global_asm;
#[cfg(target_arch = "x86_64")]
global_asm!(include_str!("asm/x86_64_syscalls.s"));

#[cfg(target_arch = "aarch64")]
global_asm!(include_str!("asm/aarch64_syscalls.s"));
