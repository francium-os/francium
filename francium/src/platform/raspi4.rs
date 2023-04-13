use crate::arch::{aarch64, arch_timer::ArchTimer, gicv2::*};
use crate::constants::*;
use crate::drivers::pl011_uart::Pl011Uart;
use crate::drivers::{InterruptController, InterruptDistributor, Timer};
use francium_common::types::PhysAddr;
use spin::Mutex;

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // 2gb for now

// uart0 is at 0x7e201000 which i think is at 0xfe201000 in low peri mode

const RPI_GICD_BASE: usize = PERIPHERAL_BASE + 0xff841000;
const RPI_GICC_BASE: usize = PERIPHERAL_BASE + 0xff842000;

lazy_static! {
    pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(
        PERIPHERAL_BASE + 0xfe201000,
        115200,
        48000000
    ));
    pub static ref INTERRUPT_CONTROLLER: Mutex<GICv2Cpu> = Mutex::new(GICv2Cpu::new(RPI_GICC_BASE));
    pub static ref INTERRUPT_DISTRIBUTOR: Mutex<GICv2Dist> =
        Mutex::new(GICv2Dist::new(RPI_GICD_BASE));
    pub static ref DEFAULT_TIMER: Mutex<ArchTimer> = Mutex::new(ArchTimer::new());
}

extern "C" {
    fn spin_for_cycles(cycle_count: usize);
}

const GPIO_BASE: usize = PERIPHERAL_BASE + 0xfe200000;

/*unsafe fn read_gpfsel0() -> u32 {
    ((GPIO_BASE + 0) as *mut u32).read_volatile()
}*/

/*unsafe fn write_gpfsel0(value: u32) {
    ((GPIO_BASE + 0x00) as *mut u32).write_volatile(value)
}*/

unsafe fn read_gpfsel1() -> u32 {
    ((GPIO_BASE + 0x04) as *mut u32).read_volatile()
}

unsafe fn write_gpfsel1(value: u32) {
    ((GPIO_BASE + 0x04) as *mut u32).write_volatile(value)
}

/*unsafe fn read_gppud() -> u32 {
    ((GPIO_BASE + 0x94) as *mut u32).read_volatile()
}*/

unsafe fn write_gppud(value: u32) {
    ((GPIO_BASE + 0x94) as *mut u32).write_volatile(value)
}

/*unsafe fn read_gppudclk0() -> u32 {
    ((GPIO_BASE + 0x98) as *mut u32).read_volatile()
}*/

unsafe fn write_gppudclk0(value: u32) {
    ((GPIO_BASE + 0x98) as *mut u32).write_volatile(value)
}

pub fn platform_specific_init() {
    /*const GPFSEL0   = 0xfe200000;
    const GPFSEL1   = 0xfe200004;
    const GPPUD     = 0xfe200094;
    const GPPUDCLK0 = 0xfe200098;*/

    // map UART0 to GPIO pins
    unsafe {
        let mut r = read_gpfsel1();
        r &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
        r |= (4 << 12) | (4 << 15); // alt0
        write_gpfsel1(r);
        write_gppud(0); // enable pins 14 and 15
        spin_for_cycles(150);

        write_gppudclk0((1 << 14) | (1 << 15));
        spin_for_cycles(150);
        write_gppudclk0(0); // flush GPIO setup
    }
}

pub fn scheduler_pre_init() {
    // enable GIC
    let timer_irq = 16 + 14; // ARCH_TIMER_NS_EL1_IRQ + 16 because "lol no u"

    let mut controller_lock = INTERRUPT_CONTROLLER.lock();
    let mut distributor_lock = INTERRUPT_DISTRIBUTOR.lock();
    controller_lock.init();
    distributor_lock.init();
    distributor_lock.enable_interrupt(timer_irq);

    // enable arch timer, 100hz
    let mut timer_lock = DEFAULT_TIMER.lock();
    timer_lock.set_period_us(10000);
    timer_lock.reset_timer();
}

pub fn scheduler_post_init() {
    DEFAULT_TIMER.lock().enable_timer();
}

use core::arch::global_asm;
global_asm!(include_str!("../arch/aarch64/asm/stub_raspi4.s"));
