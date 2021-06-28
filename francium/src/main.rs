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

extern "C" {
	static __text_start: i32;
	static __bss_end: i32;
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	println!("hello from rust!");
	println!("trying to allocate a physical frame");

	// set up physical allocator
	unsafe {
		let phys_mem_start = 0x40000000;
		let phys_mem_end = 0x40000000 + 0x20000000; // hardcoded 512MiB

		let text_start_virt = &__text_start as *const i32 as usize;
		let bss_end_virt = &__bss_end as *const i32 as usize;

		let text_start: usize = mmu::virt_to_phys(text_start_virt).0;
		let bss_end: usize = mmu::virt_to_phys(bss_end_virt).0;

		for i in (phys_mem_start .. phys_mem_end).step_by(0x1000).rev() {
			if !(i >= text_start && i <= bss_end) {
				phys_allocator::free(PhysAddr(i))
			}
		}
	}

	let phys_frame = unsafe {
		phys_allocator::alloc().unwrap()
	};

	println!("physical frame: {}", phys_frame);

	let physmap_base = 0xffffff0000000000;
	let kernel_base = 0xfffffff800000000;

	let mut page_table_root = PageTable::new();

	page_table_root.map_1gb(PhysAddr(0), physmap_base);
	page_table_root.map_1gb(PhysAddr(0x40000000), physmap_base + 0x40000000);

	for i in (0x0000000..0x1000000).step_by(0x200000) {
		page_table_root.map_2mb(PhysAddr(0x40000000 + i), kernel_base + i);
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