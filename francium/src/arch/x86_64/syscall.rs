use core::arch::asm;
use crate::arch::x86_64::svc_wrappers::SYSCALL_WRAPPERS;
#[naked]
unsafe extern "C" fn syscall_handler() {
    asm!("mov r10, rsp
		lea r9, [rip + current_thread_kernel_stack]
		mov rsp, [r9]

		push r11
		push rcx
		push r10

		// We need to save and restore the usermode (callee-save) registers here.
		push rbx
		push r12
		push r13
		push r14
		push r15
		push rbp

		lea rcx, [rip+{}]
		mov r10, [rcx + rax*8]
		call r10

		pop rbp
		pop r15
		pop r14
		pop r13
		pop r12
		pop rbx

		pop r10
		pop rcx
		pop r11
		mov rsp, r10

		sysretq
	",
        sym SYSCALL_WRAPPERS,
        options(noreturn)
    );
}

pub fn setup_syscall() {
	francium_x86::syscall::setup_syscall(syscall_handler as usize);
}
