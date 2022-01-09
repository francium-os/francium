use spin::Mutex;
use crate::PhysAddr;
use crate::drivers::pl011_uart::Pl011Uart;

lazy_static! {
	pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(PhysAddr(0x09000000), 115200));
}