use spin::Mutex;
use crate::PhysAddr;
use crate::drivers::pl011_uart::Pl011Uart;

lazy_static! {
	// Qemu doesn't care about the baud rate, but we give it one and a UART clock anyway.
	pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(PhysAddr(0x09000000), 115200, 48000000));
}

pub const PHYS_MEM_BASE: usize = 0x40000000;

pub fn platform_specific_init() {
	// nothing, for now
}

pub fn scheduler_pre_init() {
	// enable GIC
	let timer_irq = 16 + 14; // ARCH_TIMER_NS_EL1_IRQ + 16 because "lol no u"
	gicv2::init();
	gicv2::enable(timer_irq);
	//aarch64::enable_interrupts();

	// enable arch timer
	arch_timer::set_frequency_us(25000);
	arch_timer::reset_timer();
}

pub fn scheduler_post_init() {
	arch_timer::enable();
}

use core::arch::global_asm;
global_asm!(include_str!("../arch/aarch64/asm/stub_virt.s"));