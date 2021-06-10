#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[macro_use]
extern crate bitflags;

pub mod mmu;
pub mod bump_allocator;

use numtoa::NumToA;
use crate::mmu::PageTable;
use crate::mmu::PhysAddr;

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

	let kernel_base = 0xfffffff800000000;

	let mut page_table_root = PageTable::new();
	// map uart
	page_table_root.map_4k(PhysAddr(0x09000000), 0x09000000);

	for i in (0x0000000..0x1000000).step_by(0x1000) {
		page_table_root.map_4k(PhysAddr(0x40000000 + i), kernel_base + i);
	}

	for i in (0x50000000..0x51000000).step_by(0x1000) {
		page_table_root.map_4k(PhysAddr(i), i);
	}

	mmu::enable_mmu(&page_table_root);
	write_uart("hello from rust after enabling mmu!\n");
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_curr_el_spx_sync(lr: usize) -> ! {
	let mut buffer = [0u8; 20];

	write_uart("Exception!!! rust_curr_el_spx_sync!\n");
	write_uart("lr: ");
	write_uart(lr.numtoa_str(16, &mut buffer));
    loop {}
}

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}