use crate::constants::*;
use crate::PhysAddr;

pub struct Pl011Uart {
	base_address: usize,
	baud: u32
}

fn calculate_baud(_baud: u32, _uart_clock: u32) -> (u32, u32) {
	// try to do this in fixed point, because FP is evil
	(26, 3)
}

const FR_TXFULL: u32 = 1<<5;
const FR_RXEMPTY: u32 = 1<<4;

impl Pl011Uart {
	pub fn new(base_address: PhysAddr, baud: u32, uart_clock: u32) -> Pl011Uart {
		let mut uart = Pl011Uart {
			base_address: (PERIPHERAL_BASE + base_address.0),
			baud: baud
		};

		uart.init(uart_clock);
		uart
	}

	unsafe fn read_dr(&mut self) -> u8 {
		const UART_DR:   usize = 0x00;
		((self.base_address + UART_DR) as *mut u8).read_volatile()
	}

	unsafe fn write_dr(&mut self, byte: u8) {
		const UART_DR:   usize = 0x00;
		((self.base_address + UART_DR) as *mut u8).write_volatile(byte)
	}

	unsafe fn read_fr(&mut self) -> u32 {
		const UART_FR:   usize = 0x18;
		((self.base_address + UART_FR) as *mut u32).read_volatile()
	}

	unsafe fn write_ibrd(&mut self, ibrd: u32) {
		const UART_IBRD: usize = 0x24;
		((self.base_address + UART_IBRD) as *mut u32).write_volatile(ibrd)
	}

	unsafe fn write_fbrd(&mut self, fbrd: u32) {
		const UART_FBRD: usize = 0x28;
		((self.base_address + UART_FBRD) as *mut u32).write_volatile(fbrd)
	}

	unsafe fn write_lcrh(&mut self, lcrh: u32) {
		const UART_LCRH: usize = 0x2c;
		((self.base_address + UART_LCRH) as *mut u32).write_volatile(lcrh)
	}

	unsafe fn write_cr(&mut self, cr: u32) {
		const UART_CR:   usize = 0x30;
		((self.base_address + UART_CR) as *mut u32).write_volatile(cr)
	}

	// const UART_IMSC: usize = 0x38;

	unsafe fn write_icr(&mut self, icr: u32) {
		const UART_ICR:  usize = 0x44;
		((self.base_address + UART_ICR) as *mut u32).write_volatile(icr)
	}

	fn init(&mut self, uart_clock: u32) {
		let (whole, frac) = calculate_baud(self.baud, uart_clock);

		unsafe {
			// Disable by writing 0 to CR
			self.write_cr(0);
			// clear interrupts
			self.write_icr(0x7ff);
			// set baud rate
			self.write_ibrd(whole);
			self.write_fbrd(frac);
			// 8n1
			self.write_lcrh(0b11 << 5);

			// enable tx, rx, fifo
			self.write_cr(0x301);
		}
	}

	pub fn write_byte(&mut self, byte: u8) {
		unsafe {
			// Wait until TX full is 0 (TX fifo is empty)
			loop {
				if (self.read_fr() & FR_TXFULL) == 0 {
					break
				}
				// nop?
			}

			self.write_dr(byte);
		}
	}

	pub fn read_byte(&mut self) -> u8 {
		unsafe {
			loop {
				if (self.read_fr() & FR_RXEMPTY) == 0 {
					break
				}
				// nop?
			}
			self.read_dr()
		}
	}

	pub fn write_string(&mut self, a: &str) {
		for c in a.chars() {
			self.write_byte(c as u8);
		}
	}

	pub fn write_bytes(&mut self, a: &[u8]) {
		for c in a {
			self.write_byte(*c);
		}
	}
}