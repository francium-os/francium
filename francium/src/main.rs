#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate alloc;
extern crate smallvec;
extern crate elf_rs;
use elf_rs::*;

#[macro_use]
pub mod print;

pub mod mmu;
pub mod bump_allocator;
pub mod phys_allocator;
pub mod uart;
pub mod panic;
pub mod constants;
pub mod process;
pub mod arch;
pub mod memory;

use crate::mmu::PhysAddr;
use crate::mmu::PagePermission;
use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::process::Process;
use crate::constants::*;

use arch::aarch64::context::ExceptionContext;

extern "C" {
	static __text_start: i32;
	static __bss_end: i32;
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	println!("hello from rust!");

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

	{
		let page_table_root = &mut KERNEL_ADDRESS_SPACE.write().page_table;

		// map virt peripherals into physmap
		// TODO: seperate device mapping under kernel somewhere
		page_table_root.map_1gb(PhysAddr(0), PHYSMAP_BASE, PagePermission::KERNEL_RWX);

		// 1 gb is enough for anyone
		// TODO: know physical memory size
		page_table_root.map_1gb(PhysAddr(0x40000000), PHYSMAP_BASE + 0x40000000, PagePermission::KERNEL_RWX);

		// map kernel in
		unsafe {
			let text_start_virt = &__text_start as *const i32 as usize;
			let bss_end_virt = &__bss_end as *const i32 as usize;

			let kernel_length = bss_end_virt - text_start_virt;

			for i in (0x0000000..kernel_length).step_by(0x200000) {
				page_table_root.map_2mb(PhysAddr(0x40000000 + i), KERNEL_BASE + i, PagePermission::KERNEL_RWX);
			}
		}
	}
	println!("hello from rust before enabling mmu!");

	{
		let page_table_root = &KERNEL_ADDRESS_SPACE.read().page_table;
		mmu::enable_mmu(page_table_root);
	}

	println!("hello from rust after enabling mmu!");

	// Load the first process
	let mut p = {
		let page_table_root = &KERNEL_ADDRESS_SPACE.read().page_table;
		Process::new(page_table_root)
	};

	{
		p.use_pages();
	}

	let elf_buf = include_bytes!("../../cesium/target/aarch64-unknown-francium/release/cesium");
	let elf = elf_rs::Elf::from_bytes(elf_buf).unwrap();
	if let elf_rs::Elf::Elf64(e) = elf {
		for section in e.section_header_iter() {
			let sh = section.sh;
			if sh.sh_type() == SectionType::SHT_PROGBITS {
				if sh.flags().contains(SectionHeaderFlags::SHF_ALLOC) {
					if sh.flags().contains(SectionHeaderFlags::SHF_EXECINSTR) {
						p.address_space.create(sh.addr() as usize, sh.size() as usize, PagePermission::USER_RWX);
					}
					else {
						p.address_space.create(sh.addr() as usize, sh.size() as usize, PagePermission::USER_READ_WRITE);
					}
					unsafe {
						core::ptr::copy_nonoverlapping(elf_buf.as_ptr().offset(sh.offset() as isize), sh.addr() as *mut u8, sh.size() as usize);
					}
					println!("{:x?}", section);
				}
			}
		}

		let user_code_base = e.header().entry_point() as usize;
		let user_stack_base = 0x40000000;
		let user_stack_size = 0x4000;

		p.address_space.create(user_stack_base, user_stack_size, PagePermission::USER_READ_WRITE);

		p.setup_context(user_code_base, user_stack_base + user_stack_size);
		p.switch_to();
	}

	println!("hello from rust inside the ... user process. hm.");

    loop {}
}

#[no_mangle]
pub extern "C" fn rust_curr_el_spx_sync(ctx: &ExceptionContext) -> ! {
	println!("Exception!!! rust_curr_el_spx_sync!\n");
	println!("lr: {:x}, esr: {:x}", ctx.saved_pc, ctx.esr);

    loop {}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_spx_sync(ctx: &mut ExceptionContext) {
	let ec = (ctx.esr & (0x3f << 26)) >> 26;
	let iss = ctx.esr & 0xffffff;

	if ec == 0b010101 {
		if iss == 0x01 {
			let mut temp_buffer: [u8; 256] = [0; 256];
			unsafe {
				core::ptr::copy_nonoverlapping(ctx.regs[0] as *const u8, temp_buffer.as_mut_ptr(), ctx.regs[1]);
			}
			println!("[Debug] {}", core::str::from_utf8(&temp_buffer[0..ctx.regs[1]]).unwrap());
		}
	} else {
		println!("Exception!!! rust_lower_el_spx_sync!\n");
		println!("lr: {:x}, esr: {:x}", ctx.saved_pc, ctx.esr);
		loop {}
	}
}