use crate::arch::{aarch64, arch_timer::ArchTimer};
use crate::constants::*;
use crate::drivers::bcm_interrupt::*;
use crate::drivers::pl011_uart::Pl011Uart;
use crate::drivers::{InterruptController, InterruptDistributor, Timer};
use francium_common::types::PhysAddr;
use spin::Mutex;

// TODO: we need multiple interrupt controllers to do this properly
// for now: only arm local interrupts

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x3F000000 - 0x80000; // 1gbow

pub const RPI_PERIPHERAL_BASE: usize = PERIPHERAL_BASE + 0x3f000000;

lazy_static! {
    pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(
        PERIPHERAL_BASE + 0x3f201000,
        115200,
        48000000
    ));

    /*pub static ref INTERRUPT_CONTROLLER: Mutex<BCMGlobalInterruptController> =
        Mutex::new(BCMInterrupt::new(RPI_PERIPHERAL_BASE + 0xb000));*/

    pub static ref INTERRUPT_CONTROLLER: Mutex<BCMLocalInterrupt> =
        Mutex::new(BCMLocalInterrupt::new(PERIPHERAL_BASE + 0x4000_0000));

    pub static ref INTERRUPT_DISTRIBUTOR: Mutex<BCMLocalInterrupt> =
        Mutex::new(BCMLocalInterrupt::new(PERIPHERAL_BASE + 0x4000_0000));

    pub static ref DEFAULT_TIMER: Mutex<ArchTimer> = Mutex::new(ArchTimer::new());
}

extern "C" {
    fn spin_for_cycles(cycle_count: usize);
}

const GPIO_BASE: usize = PERIPHERAL_BASE + 0x3f200000;

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
    let timer_irq = 1;

    let mut controller_lock = INTERRUPT_CONTROLLER.lock();
    let mut distributor_lock = INTERRUPT_DISTRIBUTOR.lock();

    InterruptController::init(&mut *controller_lock);
    InterruptDistributor::init(&mut *distributor_lock);

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
global_asm!(include_str!("../arch/aarch64/asm/stub_raspi3.s"));
