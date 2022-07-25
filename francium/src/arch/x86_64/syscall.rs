use core::arch::asm;
use crate::arch::x86_64::msr;

#[naked]
unsafe extern "C" fn syscall_handler() {
	asm!("
		mov r10, rsp
		movabs rsp, offset syscall_stack_top
		push r11
		push rcx
		push r10

		movabs rcx, offset syscall_wrappers
		mov r10, [rcx + rax*8]
		call r10

		pop r10
		pop rcx
		pop r11
		mov rsp, r10
		sysretq
	", options(noreturn));
}

pub fn setup_syscall() {
	unsafe {
		// enable syscall instructions
		msr::write_efer(msr::read_efer() | (1<<0));

		msr::write_fmask(0); // TODO: setup rflags mask
		// kernel segment base = 0x08 (code seg = 0x08, stack seg = 0x10)
		// user segment base = 0x18 (code seg = 0x18, stack seg = 0x20)
		msr::write_star(0x08 << 32 | 0x18 << 48);
		msr::write_lstar(syscall_handler as usize); // syscall handler location
	}
}