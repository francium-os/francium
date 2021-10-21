#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(linked_list_cursors)]

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
pub mod handle;
pub mod handle_table;
pub mod mmu;
pub mod bump_allocator;
pub mod phys_allocator;
pub mod uart;
pub mod panic;
pub mod constants;
pub mod process;
pub mod arch;
pub mod memory;
pub mod scheduler;
pub mod svc;

use crate::mmu::PhysAddr;
use crate::mmu::PagePermission;
use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::memory::AddressSpace;
use crate::process::Process;
use crate::constants::*;

use crate::arch::aarch64;
use crate::arch::aarch64::gicv2;
use crate::arch::aarch64::arch_timer;

use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;

extern "C" {
	static __text_start: i32;
	static __bss_end: i32;
}

fn load_process(elf_buf: &[u8]) -> Box<Process> {
	// Load the first process
	let aspace = { 
		let page_table_root = &KERNEL_ADDRESS_SPACE.read().page_table;
		AddressSpace::new(page_table_root.user_process())
	};

	let mut p = Box::new(Process::new(Box::new(aspace)));
	p.use_pages();
	
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
		return p
	}
	panic!("Failed to load process??");
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

		let text_start: usize = text_start_virt - KERNEL_BASE + phys_mem_start;
		let bss_end: usize = bss_end_virt - KERNEL_BASE + phys_mem_start;

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
	mmu::enable_mmu();
	println!("hello from rust after enabling mmu!");

	// Set up kernel heap
	{ 
		let kernel_aspace = &mut KERNEL_ADDRESS_SPACE.write();
		kernel_aspace.create(KERNEL_HEAP_BASE, KERNEL_HEAP_INITIAL_SIZE, PagePermission::KERNEL_READ_WRITE);
	};

	// enable GIC
	let timer_irq = 16 + 14; // ARCH_TIMER_NS_EL1_IRQ + 16 because "lol no u"
	gicv2::init();
	gicv2::enable(timer_irq);
	aarch64::enable_interrupts();

	// enable arch timer
	arch_timer::set_frequency_us(25000);
	arch_timer::reset_timer();

	let elf_one_buf = include_bytes!("../../hydrogen/target/aarch64-unknown-francium/release/hydrogen");
	let elf_two_buf = include_bytes!("../../cesium/target/aarch64-unknown-francium/release/cesium");

	println!("Loading process one...");
	let proc_one = load_process(elf_one_buf);
	let proc_one_arc = Arc::new(Mutex::new(proc_one));
	scheduler::register_process(proc_one_arc.clone());

	println!("Loading process two...");
	let proc_two = load_process(elf_two_buf);
	let proc_two_arc = Arc::new(Mutex::new(proc_two));
	scheduler::register_process(proc_two_arc.clone());

	arch_timer::enable();

	println!("Running...");
	process::force_switch_to(proc_one_arc);
	println!("We shouldn't get here, ever!!");

    loop {}
}