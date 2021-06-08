#![no_std]
#![no_main]
#![feature(global_asm)]

pub fn write_uart(a: &str) {
	let uart_base: *mut u8 = 0x09000000 as *mut u8;
	for c in a.chars() {
		unsafe {
			*uart_base = c as u8;
		}
	}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	write_uart("hello from rust!\n");
    loop {}
}

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

global_asm!(include_str!("entry.s"));