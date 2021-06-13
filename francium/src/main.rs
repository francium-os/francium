#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[macro_use]
extern crate bitflags;

pub mod mmu;
pub mod bump_allocator;
pub mod phys_allocator;
pub mod uart;
pub mod panic;
pub mod print;

use crate::mmu::PageTable;
use crate::mmu::PhysAddr;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	println!("hello from rust!");
	println!("trying to allocate a physical frame");
	unsafe {
		phys_allocator::free(PhysAddr(0x40000000 + 0x100000))
	}

	let phys_frame = unsafe {
		phys_allocator::alloc().unwrap()
	};

	println!("physical frame: {}", phys_frame);

	let physmap_base = 0xffffff0000000000;
	let kernel_base = 0xfffffff800000000;

	let mut page_table_root = PageTable::new();
	// map uart

	page_table_root.map_1gb(PhysAddr(0x40000000), physmap_base + 0x40000000);
	page_table_root.map_4k(PhysAddr(0x09000000), 0x09000000);

	for i in (0x0000000..0x1000000).step_by(0x200000) {
		page_table_root.map_2mb(PhysAddr(0x40000000 + i), kernel_base + i);
	}

	for i in (0x50000000..0x51000000).step_by(0x200000) {
		page_table_root.map_2mb(PhysAddr(i), i);
	}

	mmu::enable_mmu(&page_table_root);
	println!("hello from rust after enabling mmu!");

    loop {}
}

#[no_mangle]
pub extern "C" fn rust_curr_el_spx_sync(lr: usize) -> ! {
	println!("Exception!!! rust_curr_el_spx_sync!\n");
	println!("lr: {:x}", lr);
    loop {}
}