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