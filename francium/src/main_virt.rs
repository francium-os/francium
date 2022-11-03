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

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	platform::platform_specific_init();

	let phys_mem_start = platform::PHYS_MEM_BASE;
	let phys_mem_end = platform::PHYS_MEM_BASE + platform::PHYS_MEM_SIZE;

	init::setup_physical_allocator(phys_mem_start, phys_mem_end);
	init::setup_virtual_memory();

	println!("hello from rust before enabling mmu!");
	mmu::enable_mmu();
	println!("hello from rust after enabling mmu!");

	// Set up kernel heap
	{ 
		let kernel_aspace = &mut KERNEL_ADDRESS_SPACE.write();
		kernel_aspace.create(KERNEL_HEAP_BASE, KERNEL_HEAP_INITIAL_SIZE, PagePermission::KERNEL_READ_WRITE);
	}

	platform::scheduler_pre_init();

	let fs_buf = include_bytes!("../../target/aarch64-unknown-francium/release/fs");
	let test_buf = include_bytes!("../../target/aarch64-unknown-francium/release/test");
	let sm_buf = include_bytes!("../../target/aarch64-unknown-francium/release/sm");

	let fs_main_thread = init::load_process(fs_buf, "fs");
	scheduler::register_thread(fs_main_thread.clone());

	let test_main_thread = init::load_process(test_buf, "test");
	scheduler::register_thread(test_main_thread.clone());

	let sm_main_thread = init::load_process(sm_buf, "sm");
	scheduler::register_thread(sm_main_thread.clone());

	platform::scheduler_post_init();

	println!("Running...");
	process::force_switch_to(fs_main_thread);
	println!("We shouldn't get here, ever!!");

	loop {}
}