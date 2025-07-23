#![no_std]
#![no_main]

use francium_kernel::constants::*;
use francium_kernel::memory::KERNEL_ADDRESS_SPACE;
use francium_kernel::mmu::PagePermission;
use francium_kernel::*;
use log_sink::*;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    platform::platform_specific_init();

    let phys_mem_start = platform::PHYS_MEM_BASE + 0x80000;
    let phys_mem_end = platform::PHYS_MEM_BASE + platform::PHYS_MEM_SIZE;

    init::setup_physical_allocator(phys_mem_start, phys_mem_end);
    init::setup_virtual_memory();
    init::setup_boot_per_cpu();

    println!("hello from rust before enabling mmu!");
    mmu::enable_mmu();
    println!("hello from rust after enabling mmu!");
    println!("setting up heap!");
    // Set up kernel heap
    {
        let kernel_aspace = &mut KERNEL_ADDRESS_SPACE.write();
        kernel_aspace.create(
            KERNEL_HEAP_BASE,
            KERNEL_HEAP_INITIAL_SIZE,
            PagePermission::KERNEL_READ_WRITE,
        );
    }

    println!("setup print_log_sink");

    print_log_sink::init().unwrap();

    println!("setup pre scheduler");

    platform::scheduler_pre_init();

    println!("setup scheduler");

    scheduler::init(1);
    // todo
    // platform::bringup_other_cpus();

    let fs_buf = include_bytes!("../../target/aarch64-unknown-francium/release/fs");
    let test_buf = include_bytes!("../../target/aarch64-unknown-francium/release/test");
    let sm_buf = include_bytes!("../../target/aarch64-unknown-francium/release/sm");
    let disp_buf = include_bytes!("../../target/aarch64-unknown-francium/release/disp");
    let net_buf = include_bytes!("../../target/aarch64-unknown-francium/release/net");
    let loader_buf = include_bytes!("../../target/aarch64-unknown-francium/release/loader");


    println!("loading fs...");
    let fs_main_thread = init::load_process(fs_buf, "fs");
    scheduler::register_thread(fs_main_thread.clone());

    let test_main_thread = init::load_process(test_buf, "test");
    scheduler::register_thread(test_main_thread.clone());

    let sm_main_thread = init::load_process(sm_buf, "sm");
    scheduler::register_thread(sm_main_thread.clone());

    let disp_main_thread = init::load_process(disp_buf, "disp");
    scheduler::register_thread(disp_main_thread.clone());

    let net_main_thread = init::load_process(net_buf, "net");
    scheduler::register_thread(net_main_thread.clone());

    let loader_main_thread = init::load_process(loader_buf, "loader");
    scheduler::register_thread(loader_main_thread.clone());

    platform::scheduler_post_init();

    println!("Running...");
    scheduler::force_switch_to(fs_main_thread);
    println!("We shouldn't get here, ever!!");

    loop {}
}
