use crate::arch::{arch_timer::ArchTimer, gicv2::GICv2};
use crate::constants;
use crate::drivers::pl011_uart::Pl011Uart;
use crate::drivers::InterruptController;
use crate::drivers::Timer;
use spin::Mutex;

const VIRT_GICD_BASE: usize = constants::PERIPHERAL_BASE + 0x08000000;
const VIRT_GICC_BASE: usize = constants::PERIPHERAL_BASE + 0x08010000;

lazy_static! {
    // Qemu doesn't care about the baud rate, but we give it one and a UART clock anyway.
    pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(constants::PERIPHERAL_BASE + 0x09000000, 115200, 48000000));
    pub static ref DEFAULT_INTERRUPT: Mutex<GICv2> = Mutex::new(GICv2::new(VIRT_GICD_BASE, VIRT_GICC_BASE));
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
    let gic_lock = DEFAULT_INTERRUPT.lock();
    gic_lock.init();
    gic_lock.enable_interrupt(timer_irq);

    // enable arch timer, 100hz
    let mut timer_lock = DEFAULT_TIMER.lock();
    timer_lock.set_period_us(10000);
    timer_lock.reset_timer();
}

pub fn scheduler_post_init() {
    DEFAULT_TIMER.lock().enable_timer();
}

use core::arch::global_asm;
global_asm!(include_str!("../arch/aarch64/asm/stub_virt.s"));
