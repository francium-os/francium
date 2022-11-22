use spin::Mutex;
use crate::mmu::PhysAddr;
use crate::arch::{aarch64, gicv2::GICv2, arch_timer::ArchTimer};
use crate::drivers::pl011_uart::Pl011Uart;
use crate::constants;
use crate::drivers::InterruptController;
use crate::drivers::Timer;

const VIRT_GICD_BASE: usize = constants::PERIPHERAL_BASE + 0x08000000;
const VIRT_GICC_BASE: usize = constants::PERIPHERAL_BASE + 0x08010000;

lazy_static! {
	// Qemu doesn't care about the baud rate, but we give it one and a UART clock anyway.
	pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(PhysAddr(0x09000000), 115200, 48000000));
	pub static ref GIC: Mutex<GICv2> = Mutex::new(GICv2::new(VIRT_GICD_BASE, VIRT_GICC_BASE));
	pub static ref DEFAULT_TIMER: Mutex<ArchTimer> = Mutex::new(ArchTimer::new());
}

pub const PHYS_MEM_BASE: usize = 0x40000000;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // idk 2 gig

pub fn platform_specific_init() {
	// nothing, for now
}

pub fn scheduler_pre_init() {
	// enable GIC
	let timer_irq = 16 + 14; // ARCH_TIMER_NS_EL1_IRQ + 16 because "lol no u"
	let gic_lock = GIC.lock();
	gic_lock.init();
	gic_lock.enable_interrupt(timer_irq);
	aarch64::enable_interrupts();

	// enable arch timer
	let timer_lock = DEFAULT_TIMER.lock();

	// 100Hz
	timer_lock.set_period_us(10000);
	timer_lock.reset_timer();
}

pub fn scheduler_post_init() {
	DEFAULT_TIMER.lock().enable_timer();
}

use core::arch::global_asm;
global_asm!(include_str!("../arch/aarch64/asm/stub_virt.s"));
