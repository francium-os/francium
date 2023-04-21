#![no_std]
#![no_main]

use francium_kernel::constants::*;
use francium_kernel::memory::KERNEL_ADDRESS_SPACE;
use francium_kernel::mmu::PagePermission;
use francium_kernel::*;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    platform::platform_specific_init();

    let phys_mem_start = platform::PHYS_MEM_BASE + 0x100000;
    let phys_mem_end = platform::PHYS_MEM_BASE + platform::PHYS_MEM_SIZE;

    init::setup_physical_allocator(phys_mem_start, phys_mem_end);
    init::setup_virtual_memory();
    init::setup_boot_per_cpu();

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
    scheduler::init(platform::get_cpu_count());
    // todo
    // platform::bringup_other_cpus();

    let fs_buf = include_bytes!("../../target/aarch64-unknown-francium/release/fs");
    let test_buf = include_bytes!("../../target/aarch64-unknown-francium/release/test");
    let sm_buf = include_bytes!("../../target/aarch64-unknown-francium/release/sm");
    let pcie_buf = include_bytes!("../../target/aarch64-unknown-francium/release/pcie");
    let disp_buf = include_bytes!("../../target/aarch64-unknown-francium/release/disp");

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
    scheduler::force_switch_to(disp_main_thread);
    println!("We shouldn't get here, ever!!");

    loop {}
}
