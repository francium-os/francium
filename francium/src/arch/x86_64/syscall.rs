use core::arch::asm;
use crate::arch::x86_64::msr;

#[naked]
unsafe extern "C" fn syscall_handler() {
	asm!("push r11; push rcx; xchg bx,bx" , options(noreturn));
	// TODO: the rest of the owl
}

pub fn setup_syscall() {
	unsafe {
		// enable syscall instructions
		msr::write_efer(msr::read_efer() | (1<<0));

		msr::write_fmask(0); // TODO: setup rflags mask
		msr::write_star(8 << 32); // code segment = 8
		msr::write_lstar(syscall_handler as usize); // syscall handler location
	}
}