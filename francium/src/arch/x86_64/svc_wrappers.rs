use crate::arch::x86_64::info::*;
use crate::{scheduler, svc};
use common::system_info::{SystemInfo, SystemInfoType};
use francium_common::types::PhysAddr;

// The System V ABI returns 128 bit values in rax:rdx.
// God help me if I need three return values.
#[repr(C)]
struct Pair {
    a: usize,
    b: usize,
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_break() {
    svc::svc_break();
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_debug_output(s: *const u8, length: usize) {
    svc::svc_debug_output(s, length);
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_create_port(tag: u64) -> Pair {
    let (res, out) = svc::svc_create_port(tag);
    Pair {
        a: res.0 as usize,
        b: out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_connect_to_named_port(tag: u64) -> Pair {
    let (res, out) = svc::svc_connect_to_named_port(tag);
    Pair {
        a: res.0 as usize,
        b: out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_connect_to_port_handle(handle: u32) -> Pair {
    let (res, out) = svc::svc_connect_to_port_handle(handle);
    Pair {
        a: res.0 as usize,
        b: out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_exit_process() {
    svc::svc_exit_process();
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_close_handle(handle: u32) -> u32 {
    svc::svc_close_handle(handle).0
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_ipc_request(handle: u32, ipc_buffer: usize) -> u32 {
    svc::svc_ipc_request(handle, ipc_buffer).0
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_ipc_reply(handle: u32, ipc_buffer: usize) -> u32 {
    svc::svc_ipc_reply(handle, ipc_buffer).0
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_ipc_receive(
    handles: *const u32,
    index: usize,
    ipc_buffer: usize,
) -> Pair {
    let (res, out) = svc::svc_ipc_receive(handles, index, ipc_buffer);
    Pair {
        a: res.0 as usize,
        b: out,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_ipc_accept(handle: u32) -> Pair {
    let (res, out) = svc::svc_ipc_accept(handle);
    Pair {
        a: res.0 as usize,
        b: out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_get_process_id() -> usize {
    svc::svc_get_process_id()
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_map_memory(
    address: usize,
    length: usize,
    permission: u64,
) -> Pair {
    let (res, out) = svc::svc_map_memory(address, length, permission);
    Pair {
        a: res.0 as usize,
        b: out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_sleep_ns(ns: u64) {
    svc::svc_sleep_ns(ns);
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_get_thread_id() -> usize {
    svc::svc_get_thread_id()
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_bodge(key: u32, addr: usize) -> usize {
    // just impl it here its fine :tm:
    match key {
        common::constants::GET_FS => {
            let current_thread = scheduler::get_current_thread();
            let fs = current_thread.context.lock().regs.fs;
            fs
        }
        common::constants::SET_FS => {
            let current_thread = scheduler::get_current_thread();
            current_thread.context.lock().regs.fs = addr;

            // Important: also set fs_base here, so it gets set immediately.
            crate::arch::msr::write_fs_base(addr);

            0
        }
        common::constants::GET_ACPI_BASE => SYSTEM_INFO_RSDP_ADDR.unwrap() as usize,
        _ => {
            panic!("unknown syscall_bodge key!");
        }
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_create_thread(entry_point: usize, stack_top: usize) -> Pair {
    let (res, thread_handle_out) = svc::svc_create_thread(entry_point, stack_top);
    Pair {
        a: res.0 as usize,
        b: thread_handle_out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_futex_wait(
    addr: usize,
    expected: u32,
    _timeout_ns: usize,
) -> u32 {
    svc::svc_futex_wait(addr, expected, _timeout_ns).0
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_futex_wake(addr: usize) -> u32 {
    svc::svc_futex_wake(addr).0
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_map_device_memory(
    phys_addr: PhysAddr,
    virt_addr: usize,
    length: usize,
    map_type: usize,
    permission: u64,
) -> Pair {
    let (res, out) = svc::svc_map_device_memory(phys_addr, virt_addr, length, map_type, permission);
    Pair {
        a: res.0 as usize,
        b: out as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_get_system_info(
    ty: SystemInfoType,
    index: usize,
    out_ptr: *mut SystemInfo,
) -> u32 {
    let res = svc::svc_get_system_info(ty, index, out_ptr);
    res.0 as u32
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_get_system_tick() -> u64 {
    svc::svc_get_system_tick()
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_query_physical_address(addr: usize) -> Pair {
    let (res, phys) = svc::svc_query_physical_address(addr);
    Pair {
        a: res.0 as usize,
        b: phys,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_create_event() -> Pair {
    let (res, event_handle) = svc::svc_create_event();
    Pair {
        a: res.0 as usize,
        b: event_handle as usize,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_bind_interrupt(h: u32, index: usize) -> u32 {
    let res = svc::svc_bind_interrupt(h, index);
    res.0 as u32
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_unbind_interrupt(h: u32, index: usize) -> u32 {
    let res = svc::svc_unbind_interrupt(h, index);
    res.0 as u32
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_wait_one(h: u32) -> u32 {
    let res = svc::svc_wait_one(h);
    res.0 as u32
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_signal_event(h: u32) -> u32 {
    let res = svc::svc_signal_event(h);
    res.0 as u32
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_clear_event(h: u32) -> u32 {
    let res = svc::svc_clear_event(h);
    res.0 as u32
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_wait_many(handles: *const u32, index: usize) -> Pair {
    let (res, out) = svc::svc_wait_many(handles, index);
    Pair {
        a: res.0 as usize,
        b: out,
    }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_create_session(
    server_session_out: *mut u32,
    client_session_out: *mut u32,
) -> u32 {
    let res = svc::svc_create_session(server_session_out, client_session_out);
    res.0 as u32
}

// Don't modify this. Honest.
pub static mut SYSCALL_WRAPPERS: [*const usize; 31] = [
    syscall_wrapper_break as *const usize,
    syscall_wrapper_debug_output as *const usize,
    syscall_wrapper_create_port as *const usize,
    syscall_wrapper_connect_to_named_port as *const usize,
    syscall_wrapper_exit_process as *const usize,
    syscall_wrapper_close_handle as *const usize,
    syscall_wrapper_ipc_request as *const usize,
    syscall_wrapper_ipc_reply as *const usize,
    syscall_wrapper_ipc_receive as *const usize,
    syscall_wrapper_ipc_accept as *const usize,
    syscall_wrapper_get_process_id as *const usize,
    syscall_wrapper_connect_to_port_handle as *const usize,
    syscall_wrapper_map_memory as *const usize,
    syscall_wrapper_sleep_ns as *const usize,
    syscall_wrapper_bodge as *const usize,
    syscall_wrapper_get_thread_id as *const usize,
    syscall_wrapper_create_thread as *const usize,
    syscall_wrapper_futex_wait as *const usize,
    syscall_wrapper_futex_wake as *const usize,
    syscall_wrapper_map_device_memory as *const usize,
    syscall_wrapper_get_system_info as *const usize,
    syscall_wrapper_get_system_tick as *const usize,
    syscall_wrapper_query_physical_address as *const usize,
    syscall_wrapper_create_event as *const usize,
    syscall_wrapper_bind_interrupt as *const usize,
    syscall_wrapper_unbind_interrupt as *const usize,
    syscall_wrapper_wait_one as *const usize,
    syscall_wrapper_signal_event as *const usize,
    syscall_wrapper_clear_event as *const usize,
    syscall_wrapper_wait_many as *const usize,
    syscall_wrapper_create_session as *const usize,
];
