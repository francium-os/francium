use core::arch::asm;

#[naked]
unsafe extern "C" fn syscall_handler() {
    asm!(
        "
		mov r10, rsp
		movabs r9, offset current_thread_kernel_stack
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

		movabs rcx, offset syscall_wrappers
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
        options(noreturn)
    );
}

pub fn setup_syscall() {
	francium_x86::syscall::setup_syscall(syscall_handler as usize);
}
