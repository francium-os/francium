use crate::arch::context::ExceptionContext;
use crate::svc;

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
	let res = svc::svc_ipc_request(ctx.regs[0] as u32);
	ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_ipc_reply(ctx: &mut ExceptionContext) {
	let res = svc::svc_ipc_reply(ctx.regs[0] as u32);
	ctx.regs[0] = res.0 as usize;
}

fn syscall_wrapper_ipc_receive(ctx: &mut ExceptionContext) {
	let (res, index_out) = svc::svc_ipc_receive(ctx.regs[0] as *const u32, ctx.regs[1]);
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
	let (res, addr_out) = svc::svc_map_memory(ctx.regs[0] as usize, ctx.regs[1] as usize, ctx.regs[2] as u32);
	ctx.regs[0] = res.0 as usize;
	ctx.regs[1] = addr_out;
}

fn syscall_wrapper_sleep_ns(ctx: &mut ExceptionContext) {
	svc::svc_sleep_ns(ctx.regs[0] as u64);
}

type SVCHandler = fn(&mut ExceptionContext);
pub const SVC_HANDLERS: [SVCHandler; 14] = [
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
	syscall_wrapper_sleep_ns
];