#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]

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

use francium_drivers as drivers;

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
pub mod acpi;

use francium_common::types::PhysAddr;
use crate::constants::*;
use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::mmu::PagePermission;

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

    unsafe {
        switch_stacks();
    }
    bootloader_main(info);
}

static mut RSDP_ADDRESS: Option<u64> = None;

#[cfg(feature = "platform_pc")]
fn bootloader_main(info: &'static mut bootloader::BootInfo) -> ! {
    platform::platform_specific_init();
    let rsdp_addr = info.rsdp_addr.into_option().unwrap();

    for m in info.memory_regions.iter() {
        println!("{:x?}", m);
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

    /* Be careful - bootloader memory mappings are clobbered when we switch. */
    unsafe {
        //FRAMEBUFFER = info.framebuffer;
        RSDP_ADDRESS = info.rsdp_addr.into_option();
    }

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

    let rsdp_phys = PhysAddr(rsdp_addr as usize);
    let rsdp = acpi::parse_rsdp(rsdp_phys);
    match rsdp {
        acpi::RSDP::Normal(rsdp) => {
            let _rsdt = acpi::parse_rsdt(PhysAddr(rsdp.rsdt_address as usize));
        },
        acpi::RSDP::Extended(_xsdp) => {
            unimplemented!();
        }
    }

    subscriber::init();

    platform::scheduler_pre_init();
    scheduler::init();

    let fs_buf = include_bytes!("../../target/x86_64-unknown-francium/release/fs");
    let test_buf = include_bytes!("../../target/x86_64-unknown-francium/release/test");
    let sm_buf = include_bytes!("../../target/x86_64-unknown-francium/release/sm");
    let pcie_buf = include_bytes!("../../target/x86_64-unknown-francium/release/pcie");

    let fs_main_thread = init::load_process(fs_buf, "fs");
    scheduler::register_thread(fs_main_thread.clone());

    let test_main_thread = init::load_process(test_buf, "test");
    scheduler::register_thread(test_main_thread.clone());

    let sm_main_thread = init::load_process(sm_buf, "sm");
    scheduler::register_thread(sm_main_thread.clone());

    let pcie_main_thread = init::load_process(pcie_buf, "pcie");
    scheduler::register_thread(pcie_main_thread.clone());

    platform::scheduler_post_init();

    println!("Running...");

    scheduler::force_switch_to(fs_main_thread);
    println!("We shouldn't get here, ever!!");

    loop {}
}
