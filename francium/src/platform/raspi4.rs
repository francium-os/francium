use spin::Mutex;
use crate::constants::*;
use crate::PhysAddr;
use crate::drivers::pl011_uart::Pl011Uart;

pub const PHYS_MEM_BASE: usize = 0;

// uart0 is at 0x7e201000 which i think is at 0xfe201000 in low peri mode

lazy_static! {
	pub static ref DEFAULT_UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(PhysAddr(0xfe201000), 115200, 48000000));
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
	    r &= !((7<<12)|(7<<15)); // gpio14, gpio15
	    r |= (4<<12)|(4<<15);    // alt0
	    write_gpfsel1(r);
	    write_gppud(0); // enable pins 14 and 15
	    spin_for_cycles(150);

	    write_gppudclk0((1<<14)|(1<<15));
	    spin_for_cycles(150);
	    write_gppudclk0(0);          // flush GPIO setup
	}
}