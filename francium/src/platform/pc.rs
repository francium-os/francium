use core::arch::asm;
use crate::arch::msr;
use crate::drivers::pc_uart::COMPort;
use crate::drivers::pit_timer::PITTimer;
use spin::Mutex;

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // 2gb?? for now

unsafe fn turn_on_floating_point() {
	core::arch::asm!("
		mov rax, cr0
		and ax, 0xFFFB
		or ax, 0x2
		mov cr0, rax
		mov rax, cr4
		or ax, 3 << 9
		mov cr4, rax
	");
}

lazy_static! {
	pub static ref DEFAULT_UART: Mutex<COMPort> = Mutex::new(COMPort::new(0x3f8));
	pub static ref DEFAULT_TIMER: Mutex<PITTimer> = Mutex::new(PITTimer::new());
}

pub fn platform_specific_init() {
}

pub fn scheduler_pre_init() {
	//unimplemented!();
}

pub fn scheduler_post_init() {
	unsafe { turn_on_floating_point(); }

	// XXX give this a constant
	// Enable XN
	unsafe { msr::write_efer(msr::read_efer() | (1<<11)); }
}
