use core::arch::asm;
use crate::arch::x86_64::svc_wrappers::SYSCALL_WRAPPERS;
#[naked]
unsafe extern "C" fn syscall_handler() {
    asm!("mov r9, rsp
		mov rsp, [rip + current_thread_kernel_stack]

		push r11
		push rcx
		push r9

		// We need to save and restore the usermode (callee-save) registers here.
		push rbx
		push r12
		push r13
		push r14
		push r15
		push rbp

		mov rcx, r10

		lea r11, [rip+{}]
		mov r11, [r11 + rax*8]
		call r11

		pop rbp
		pop r15
		pop r14
		pop r13
		pop r12
		pop rbx

		pop r9
		pop rcx
		pop r11
		mov rsp, r9

		sysretq
	",
        sym SYSCALL_WRAPPERS,
        options(noreturn)
    );
}

pub fn setup_syscall() {
	francium_x86::syscall::setup_syscall(syscall_handler as usize);
}
