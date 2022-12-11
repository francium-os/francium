#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate alloc;
extern crate elf_rs;
extern crate smallvec;

#[macro_use]
pub mod print;

pub mod align;
pub mod constants;
pub mod drivers;
pub mod panic;
pub mod platform;

pub mod bump_allocator;
pub mod handle;
pub mod handle_table;
pub mod mmu;
pub mod phys_allocator;

pub mod arch;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod svc;
pub mod timer;
pub mod waitable;

pub mod init;

pub mod subscriber;

use crate::constants::*;
use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::mmu::PagePermission;

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
        kernel_aspace.create(
            KERNEL_HEAP_BASE,
            KERNEL_HEAP_INITIAL_SIZE,
            PagePermission::KERNEL_READ_WRITE,
        );
    }

    // Now we can create the tracing subscriber, and also set up the idle process.

    subscriber::init();

    platform::scheduler_pre_init();
    scheduler::init();

    let fs_buf = include_bytes!("../../target/aarch64-unknown-francium/release/fs");
    let test_buf = include_bytes!("../../target/aarch64-unknown-francium/release/test");
    let sm_buf = include_bytes!("../../target/aarch64-unknown-francium/release/sm");

    let fs_main_thread = init::load_process(fs_buf, "fs");
    scheduler::register_thread(fs_main_thread.clone());

    let test_main_thread = init::load_process(test_buf, "test");
    scheduler::register_thread(test_main_thread);

    let sm_main_thread = init::load_process(sm_buf, "sm");
    scheduler::register_thread(sm_main_thread);

    platform::scheduler_post_init();

    println!("Running...");
    scheduler::force_switch_to(fs_main_thread);
    println!("We shouldn't get here, ever!!");

    loop {}
}
