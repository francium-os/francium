use core::arch::global_asm;
use crate::{svc, scheduler};

// The System V ABI returns 128 bit values in rax:rdx.
// God help me if I need three return values.
#[repr(C)]
struct Pair {
	a: usize,
	b: usize
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
	Pair { a: res.0 as usize, b: out as usize }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_connect_to_named_port(tag: u64) -> Pair {
	let (res, out) = svc::svc_connect_to_named_port(tag);
	Pair { a: res.0 as usize, b: out as usize }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_connect_to_port_handle(handle: u32) -> Pair {
	let (res, out) = svc::svc_connect_to_port_handle(handle);
	Pair { a: res.0 as usize, b: out as usize }
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
unsafe extern "C" fn syscall_wrapper_ipc_receive(handles: *const u32, index: usize, ipc_buffer: usize) -> Pair {
	let (res, out) = svc::svc_ipc_receive(handles, index, ipc_buffer);
	Pair { a: res.0 as usize, b: out }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_ipc_accept(handle: u32) -> Pair {
	let (res, out) = svc::svc_ipc_accept(handle);
	Pair { a: res.0 as usize, b: out as usize }
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_get_process_id() -> usize {
	svc::svc_get_process_id()
}

#[no_mangle]
unsafe extern "C" fn syscall_wrapper_map_memory(address: usize, length: usize, permission: u32) -> Pair {
	let (res, out) = svc::svc_map_memory(address, length, permission);
	Pair { a: res.0 as usize, b: out as usize }
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
		},
		common::constants::SET_FS => {
			let current_thread = scheduler::get_current_thread();
			current_thread.context.lock().regs.fs = addr;

			// Important: also set fs_base here, so it gets set immediately.
			crate::arch::msr::write_fs_base(addr);

			0
		},
		_ => {
			panic!("unknown syscall_bodge key!");
		}
	}
}

// Rust complains loudly about this. As it should.
global_asm!("
.global syscall_wrappers
syscall_wrappers:
.quad syscall_wrapper_break
.quad syscall_wrapper_debug_output
.quad syscall_wrapper_create_port
.quad syscall_wrapper_connect_to_named_port
.quad syscall_wrapper_exit_process
.quad syscall_wrapper_close_handle
.quad syscall_wrapper_ipc_request
.quad syscall_wrapper_ipc_reply
.quad syscall_wrapper_ipc_receive
.quad syscall_wrapper_ipc_accept
.quad syscall_wrapper_get_process_id
.quad syscall_wrapper_connect_to_port_handle
.quad syscall_wrapper_map_memory
.quad syscall_wrapper_sleep_ns
.quad syscall_wrapper_bodge
.quad syscall_wrapper_get_thread_id
");
