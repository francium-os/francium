use crate::arch::context::ExceptionContext;
use crate::svc;
use common::system_info::{SystemInfo, SystemInfoType};
use core::convert::TryFrom;
use francium_common::types::PhysAddr;

fn syscall_wrapper_break(_ctx: &mut ExceptionContext) {
    svc::svc_break();
}

fn syscall_wrapper_debug_output(ctx: &mut ExceptionContext) {
    svc::svc_debug_output(ctx.regs[0] as *const u8, ctx.regs[1]);
}

fn syscall_wrapper_create_port(ctx: &mut ExceptionContext) {
    let (result, handle_out) = svc::svc_create_port(ctx.regs[0] as u64);
    ctx.regs[0] = result.0 as usize;
    ctx.regs[1] = handle_out as usize;
}

fn syscall_wrapper_connect_to_named_port(ctx: &mut ExceptionContext) {
    let (result, handle_out) = svc::svc_connect_to_named_port(ctx.regs[0] as u64);
    ctx.regs[0] = result.0 as usize;
    ctx.regs[1] = handle_out as usize;
}

fn syscall_wrapper_connect_to_port_handle(ctx: &mut ExceptionContext) {
    let (result, handle_out) = svc::svc_connect_to_port_handle(ctx.regs[0] as u32);
    ctx.regs[0] = result.0 as usize;
    ctx.regs[1] = handle_out as usize;
}

fn syscall_wrapper_exit_process(_ctx: &mut ExceptionContext) {
    svc::svc_exit_process();
}

fn syscall_wrapper_close_handle(ctx: &mut ExceptionContext) {
    let res = svc::svc_close_handle(ctx.regs[0] as u32);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_ipc_request(ctx: &mut ExceptionContext) {
    let res = svc::svc_ipc_request(ctx.regs[0] as u32, ctx.regs[1] as usize);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_ipc_reply(ctx: &mut ExceptionContext) {
    let res = svc::svc_ipc_reply(ctx.regs[0] as u32, ctx.regs[1] as usize);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_ipc_receive(ctx: &mut ExceptionContext) {
    let (res, index_out) =
        svc::svc_ipc_receive(ctx.regs[0] as *const u32, ctx.regs[1], ctx.regs[2] as usize);
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = index_out;
}

fn syscall_wrapper_ipc_accept(ctx: &mut ExceptionContext) {
    let (res, handle_out) = svc::svc_ipc_accept(ctx.regs[0] as u32);
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = handle_out as usize;
}

fn syscall_wrapper_get_process_id(ctx: &mut ExceptionContext) {
    let pid = svc::svc_get_process_id();
    ctx.regs[0] = pid;
}

fn syscall_wrapper_map_memory(ctx: &mut ExceptionContext) {
    let (res, addr_out) = svc::svc_map_memory(
        ctx.regs[0] as usize,
        ctx.regs[1] as usize,
        ctx.regs[2] as u64,
    );
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = addr_out;
}

fn syscall_wrapper_sleep_ns(ctx: &mut ExceptionContext) {
    svc::svc_sleep_ns(ctx.regs[0] as u64);
}

fn syscall_wrapper_get_thread_id(ctx: &mut ExceptionContext) {
    let tid = svc::svc_get_thread_id();
    ctx.regs[0] = tid;
}

fn syscall_wrapper_create_thread(ctx: &mut ExceptionContext) {
    let (res, tid_out) = svc::svc_create_thread(ctx.regs[0], ctx.regs[1]);
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = tid_out as usize;
}

fn syscall_wrapper_futex_wait(ctx: &mut ExceptionContext) {
    let res = svc::svc_futex_wait(ctx.regs[0], ctx.regs[1] as u32, ctx.regs[2]);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_futex_wake(ctx: &mut ExceptionContext) {
    let res = svc::svc_futex_wake(ctx.regs[0]);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_map_device_memory(ctx: &mut ExceptionContext) {
    //phys_addr: PhysAddr, virt_addr: usize, length: usize, map_type: usize, permission: u32) -> Pair {
    let (res, out) = svc::svc_map_device_memory(
        PhysAddr(ctx.regs[0]),
        ctx.regs[1],
        ctx.regs[2],
        ctx.regs[3] as usize,
        ctx.regs[4] as u64,
    );
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = out as usize;
}

fn syscall_wrapper_get_system_info(ctx: &mut ExceptionContext) {
    /* ty: usize, index: usize, out_ptr: *mut usize */
    let res = svc::svc_get_system_info(
        SystemInfoType::try_from(ctx.regs[0]).unwrap(),
        ctx.regs[1],
        ctx.regs[2] as *mut SystemInfo,
    );
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_get_system_tick(ctx: &mut ExceptionContext) {
    let res = svc::svc_get_system_tick();
    ctx.regs[0] = res as usize;
}

fn syscall_wrapper_query_physical_address(ctx: &mut ExceptionContext) {
    let (res, addr) = svc::svc_query_physical_address(ctx.regs[0]);
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = addr;
}

fn syscall_wrapper_create_event(ctx: &mut ExceptionContext) {
    let (res, handle_out) = svc::svc_create_event();
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = handle_out as usize;
}

fn syscall_wrapper_bind_interrupt(ctx: &mut ExceptionContext) {
    let res = svc::svc_bind_interrupt(ctx.regs[0] as u32, ctx.regs[1]);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_unbind_interrupt(ctx: &mut ExceptionContext) {
    let res = svc::svc_unbind_interrupt(ctx.regs[0] as u32, ctx.regs[1]);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_wait_one(ctx: &mut ExceptionContext) {
    let res = svc::svc_wait_one(ctx.regs[0] as u32);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_signal_event(ctx: &mut ExceptionContext) {
    let res = svc::svc_signal_event(ctx.regs[0] as u32);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_clear_event(ctx: &mut ExceptionContext) {
    let res = svc::svc_clear_event(ctx.regs[0] as u32);
    ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_wait_many(ctx: &mut ExceptionContext) {
    let (res, index_out) = svc::svc_wait_many(ctx.regs[0] as *const u32, ctx.regs[1] as usize);
    ctx.regs[0] = res.0 as usize;
    ctx.regs[1] = index_out;
}

fn syscall_wrapper_create_session(ctx: &mut ExceptionContext) {
    let res = svc::svc_create_session(ctx.regs[0] as *mut u32, ctx.regs[1] as *mut u32);
    ctx.regs[0] = res.0 as usize;
}

type SVCHandler = fn(&mut ExceptionContext);
pub const SVC_HANDLERS: [SVCHandler; 31] = [
    syscall_wrapper_break,
    syscall_wrapper_debug_output,
    syscall_wrapper_create_port,
    syscall_wrapper_connect_to_named_port,
    syscall_wrapper_exit_process,
    syscall_wrapper_close_handle,
    syscall_wrapper_ipc_request,
    syscall_wrapper_ipc_reply,
    syscall_wrapper_ipc_receive,
    syscall_wrapper_ipc_accept,
    syscall_wrapper_get_process_id,
    syscall_wrapper_connect_to_port_handle,
    syscall_wrapper_map_memory,
    syscall_wrapper_sleep_ns,
    syscall_wrapper_break, // unused
    syscall_wrapper_get_thread_id,
    syscall_wrapper_create_thread,
    syscall_wrapper_futex_wait,
    syscall_wrapper_futex_wake,
    syscall_wrapper_map_device_memory,
    syscall_wrapper_get_system_info,
    syscall_wrapper_get_system_tick,
    syscall_wrapper_query_physical_address,
    syscall_wrapper_create_event,
    syscall_wrapper_bind_interrupt,
    syscall_wrapper_unbind_interrupt,
    syscall_wrapper_wait_one,
    syscall_wrapper_signal_event,
    syscall_wrapper_clear_event,
    syscall_wrapper_wait_many,
    syscall_wrapper_create_session,
];
