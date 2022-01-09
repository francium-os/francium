use spin::Mutex;
use crate::PhysAddr;
use crate::drivers::pl011_uart::Pl011Uart;

// uart0 is at 0x7e201000 which i think is at 0xfe201000 in low peri mode

lazy_static! {
	pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(PhysAddr(0xfe201000), 115200));
}