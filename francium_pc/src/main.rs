#![no_std]
#![no_main]
#![feature(allocator_api)]

extern crate alloc;

use francium_kernel::constants::*;
use francium_kernel::memory::KERNEL_ADDRESS_SPACE;
use francium_kernel::mmu::PagePermission;
use francium_kernel::*;

extern "C" {
    fn switch_stacks();
    static __bootstrap_stack_top: i32;
}

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::FixedAddress(
        0xffff_f000_0000_0000,
    ));
    config
};
bootloader_api::entry_point!(bootloader_main_thunk, config = &CONFIG);

fn bootloader_main_thunk(info: &'static mut bootloader_api::BootInfo) -> ! {
    // TODO: uh, not this, please.
    // I think we need a thunk so that locals don't get allocated in the wrong stack. Maybe.
    // This is probably some kind of undefined.

    unsafe {
        switch_stacks();
    }
    bootloader_main(info);
}

fn bootloader_main(info: &'static mut bootloader_api::BootInfo) -> ! {
    platform::platform_specific_init();

    for m in info.memory_regions.iter() {
        if m.kind == bootloader_api::info::MemoryRegionKind::Usable {
            println!("using {:?} for memory", m);
            init::setup_physical_allocator(m.start as usize, m.end as usize);
        }
    }

    println!("hello from rust before setting up anything!");
    init::setup_virtual_memory();
    arch::gdt::setup_gdt();
    arch::idt::setup_idt();
    init::setup_boot_per_cpu();
    arch::syscall::setup_syscall();

    /* Be careful - bootloader memory mappings are clobbered when we switch. */
    unsafe {
        //FRAMEBUFFER = info.framebuffer;
        francium_kernel::arch::x86_64::info::SYSTEM_INFO_RSDP_ADDR = info.rsdp_addr.into_option();
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

    log_sink::init().unwrap();

    platform::scheduler_pre_init();
    scheduler::init();

    let fs_buf = include_bytes!("../../target/x86_64-unknown-francium/release/fs");
    let test_buf = include_bytes!("../../target/x86_64-unknown-francium/release/test");
    let sm_buf = include_bytes!("../../target/x86_64-unknown-francium/release/sm");
    let pcie_buf = include_bytes!("../../target/x86_64-unknown-francium/release/pcie");
    let disp_buf = include_bytes!("../../target/x86_64-unknown-francium/release/disp");

    let fs_main_thread = init::load_process(fs_buf, "fs");
    scheduler::register_thread(fs_main_thread.clone());

    let test_main_thread = init::load_process(test_buf, "test");
    scheduler::register_thread(test_main_thread.clone());

    let sm_main_thread = init::load_process(sm_buf, "sm");
    scheduler::register_thread(sm_main_thread.clone());

    let pcie_main_thread = init::load_process(pcie_buf, "pcie");
    scheduler::register_thread(pcie_main_thread.clone());

    let disp_main_thread = init::load_process(disp_buf, "disp");
    scheduler::register_thread(disp_main_thread.clone());

    platform::scheduler_post_init();

    println!("Running...");

    scheduler::force_switch_to(fs_main_thread);
    println!("We shouldn't get here, ever!!");

    loop {}
}
