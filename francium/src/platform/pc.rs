use crate::arch::msr;
use crate::drivers::pc_uart::COMPort;
use crate::drivers::pic_interrupt_controller::PIC;
use crate::drivers::pit_timer::PIT;
use crate::drivers::InterruptController;
use crate::drivers::Timer;
use core::arch::asm;
use spin::Mutex;

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // 2gb?? for now

unsafe fn turn_on_floating_point() {
    asm!(
        "
		mov rax, cr0
		and ax, 0xFFFB
		or ax, 0x2
		mov cr0, rax
		mov rax, cr4
		or ax, 3 << 9
		mov cr4, rax
	"
    );
}

lazy_static! {
    pub static ref DEFAULT_UART: Mutex<COMPort> = Mutex::new(COMPort::new(0x3f8));
    pub static ref DEFAULT_TIMER: Mutex<PIT> = Mutex::new(PIT::new());
    pub static ref DEFAULT_INTERRUPT: Mutex<PIC> = Mutex::new(PIC::new());
}

pub fn platform_specific_init() {}

pub fn scheduler_pre_init() {
    // enable timer irq
    let timer_irq = 0; // PIC on IRQ 0!
    let mut gic_lock = DEFAULT_INTERRUPT.lock();
    gic_lock.init();
    gic_lock.enable_interrupt(timer_irq);

    // enable arch timer, 100hz
    let mut timer_lock = DEFAULT_TIMER.lock();
    timer_lock.set_period_us(10000);
    timer_lock.reset_timer();
}

pub fn scheduler_post_init() {
    unsafe {
        turn_on_floating_point();
    }

    // XXX give this a constant
    // Enable XN
    unsafe {
        msr::write_efer(msr::read_efer() | (1 << 11));
    }
}
