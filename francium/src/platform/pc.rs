use crate::arch::msr;
use crate::drivers::pc_uart::COMPort;
use crate::drivers::pc_local_apic::LocalApic;
use crate::drivers::pic_interrupt_controller::{PIC, PICDist};
use crate::drivers::pit_timer::PIT;
use crate::drivers::{InterruptController, InterruptDistributor};
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
    
    pub static ref INTERRUPT_CONTROLLER: Mutex<PIC> = Mutex::new(PIC::new());
    pub static ref INTERRUPT_DISTRIBUTOR: Mutex<PICDist> = Mutex::new(PICDist::new());

    //pub static ref INTERRUPT_CONTROLLER: Mutex<LocalApic> = Mutex::new(LocalApic::new(crate::constants::PERIPHERAL_BASE + 0xFEE00000));
    //pub static ref INTERRUPT_DISTRIBUTOR: Mutex<IoApic> = Mutex::new(IoApic::new());
}

pub fn platform_specific_init() {}

pub fn scheduler_pre_init() {
    // enable timer irq
    let timer_irq = 0; // PIC on IRQ 0!

    let mut picc_lock = INTERRUPT_CONTROLLER.lock();
    let mut picd_lock = INTERRUPT_DISTRIBUTOR.lock();
    picc_lock.init();
    picd_lock.init();
    picd_lock.enable_interrupt(timer_irq);

    // Enable IRQ2 so cascading works later.
    picd_lock.enable_interrupt(2);

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
