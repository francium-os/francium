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
pub mod constants;
pub mod process;
pub mod arch;

use crate::mmu::PageTable;
use crate::mmu::PhysAddr;
use crate::process::Process;
use crate::constants::*;

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
		// TODO: know physical memory base
		let phys_mem_start = 0x40000000;

		// TODO: know physical memory size
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

	let mut page_table_root = PageTable::new();

	// map virt peripherals into physmap
	// TODO: seperate device mapping under kernel somewhere
	page_table_root.map_1gb(PhysAddr(0), PHYSMAP_BASE);

	// 1 gb is enough for anyone
	// TODO: know physical memory size
	page_table_root.map_1gb(PhysAddr(0x40000000), PHYSMAP_BASE + 0x40000000);

	// map kernel in
	unsafe {
		let text_start_virt = &__text_start as *const i32 as usize;
		let bss_end_virt = &__bss_end as *const i32 as usize;

		let kernel_length = bss_end_virt - text_start_virt;

		for i in (0x0000000..kernel_length).step_by(0x200000) {
			page_table_root.map_2mb(PhysAddr(0x40000000 + i), KERNEL_BASE + i);
		}
	}

	mmu::enable_mmu(&page_table_root);
	println!("hello from rust after enabling mmu!");

	// Load the first process
	let mut p = Process::new(&page_table_root);

	// Give it some memory... just a little
	unsafe {
		let page: PhysAddr = phys_allocator::alloc().unwrap();
		let page_two: PhysAddr = phys_allocator::alloc().unwrap();
		println!("code page {}", page);
		println!("stack page {}", page);
		p.pages.map_4k(page, 0x1000_0000);
		p.pages.map_4k(page_two, 0x4000_0000);

		phys_allocator::write_phys::<u32>(page, 0x14000000);
	}

	p.setup_context(0x1000_0000, 0x4000_0000 + 0x1000);
	p.switch_to();

	println!("hello from rust inside the ... user process. hm.");

    loop {}
}

#[no_mangle]
pub extern "C" fn rust_curr_el_spx_sync(lr: usize) -> ! {
	println!("Exception!!! rust_curr_el_spx_sync!\n");
	println!("lr: {:x}", lr);
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_spx_sync(lr: usize, esr: usize, far: usize) -> ! {
	println!("Exception!!! rust_lower_el_spx_sync!\n");
	println!("lr: {:x}, esr: {:x}, far: {:x}", lr, esr, far);

	println!("EC: {:b}", (esr & (0x3f << 26)) >> 26);
    loop {}
}