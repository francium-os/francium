#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate alloc;
extern crate smallvec;
extern crate elf_rs;

pub mod constants;
pub mod drivers;
pub mod platform;
pub mod panic;

#[macro_use]
pub mod print;

pub mod handle;
pub mod handle_table;
pub mod mmu;
pub mod bump_allocator;
pub mod phys_allocator;

pub mod process;
pub mod arch;
pub mod memory;
pub mod scheduler;
pub mod waitable;
pub mod svc;

pub mod init;

use crate::constants::*;
use crate::mmu::PagePermission;
use crate::memory::KERNEL_ADDRESS_SPACE;
use core::arch::asm;

extern "C" {
	fn switch_stacks();
	static __bootstrap_stack_top: i32;
}

#[cfg(feature = "platform_pc")]
bootloader::entry_point!(bootloader_main_thunk);

#[cfg(feature = "platform_pc")]
fn bootloader_main_thunk(info: &'static mut bootloader::BootInfo) -> ! {
	// TODO: uh, not this, please.
	// I think we need a thunk so that locals don't get allocated in the wrong stack. Maybe.
	// This is probably some kind of undefined.

	unsafe { switch_stacks(); }
	bootloader_main(info);
}

#[cfg(feature = "platform_pc")]
fn bootloader_main(info: &'static mut bootloader::BootInfo) -> ! {
	platform::platform_specific_init();

	for m in info.memory_regions.iter() {
		if m.kind == bootloader::boot_info::MemoryRegionKind::Usable {
			println!("using {:?} for memory", m);
			init::setup_physical_allocator(m.start as usize, m.end as usize);
		}
	}
 
	println!("hello from rust before setting up anything!");	
	init::setup_virtual_memory();
	arch::gdt::setup_gdt();
	arch::idt::setup_idt();
	arch::syscall::setup_syscall();

	println!("hello from rust before enabling mmu!");
	mmu::enable_mmu();
	println!("hello from rust after enabling mmu!");

	// Set up kernel heap
	{ 
		let kernel_aspace = &mut KERNEL_ADDRESS_SPACE.write();
		kernel_aspace.create(KERNEL_HEAP_BASE, KERNEL_HEAP_INITIAL_SIZE, PagePermission::KERNEL_READ_WRITE);
	}

	platform::scheduler_pre_init();

	let elf_one_buf = include_bytes!("../../target/x86_64-unknown-francium-user/release/fs");
	let elf_two_buf = include_bytes!("../../target/x86_64-unknown-francium-user/release/test");

	println!("Loading process one...");
	let one_main_thread = init::load_process(elf_one_buf);
	scheduler::register_thread(one_main_thread.clone());

	println!("Loading process two...");
	let two_main_thread = init::load_process(elf_two_buf);
	scheduler::register_thread(two_main_thread.clone());

	platform::scheduler_post_init();

	println!("Running...");
	process::force_switch_to(one_main_thread);
	println!("We shouldn't get here, ever!!");

	loop {}
}
