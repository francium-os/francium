#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(naked_functions)]

extern crate alloc;

use francium_kernel::constants::*;
use francium_kernel::memory::KERNEL_ADDRESS_SPACE;
use francium_kernel::mmu::PagePermission;
use francium_kernel::*;
use francium_kernel::arch::x86_64;
use francium_kernel::log_sink::early_framebuffer;
use francium_kernel::log_sink::early_framebuffer::{EarlyFramebuffer, EarlyFramebufferFormat, EarlyFramebufferLogger};

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
    init::setup_boot_per_cpu();

    arch::gdt::setup_gdt();
    arch::idt::setup_idt();
    arch::syscall::setup_syscall();

    /* Be careful - bootloader memory mappings are clobbered when we switch. */
    unsafe {
        let framebuffer = info.framebuffer.as_mut().unwrap();
        let fb_info = framebuffer.info();

        // Get the physical address of the framebuffer...
        let pages = arch::mmu::get_current_page_table();
        let framebuffer_slice = framebuffer.buffer_mut();
        let framebuffer_phys = pages.virt_to_phys(framebuffer_slice.as_ptr() as usize).unwrap().0;
        let framebuffer_slice_phys = core::slice::from_raw_parts_mut((constants::PHYSMAP_BASE + framebuffer_phys) as *mut u8, framebuffer_slice.len());

        // The framebuffer is probably not contained in the bootloader's mapping of memory.
        // Don't log anything before we switch page tables.
        early_framebuffer::init(early_framebuffer::EarlyFramebuffer {
                framebuffer: framebuffer_slice_phys,
                width: fb_info.width,
                height: fb_info.height,
                stride: fb_info.stride,
                bytes_per_pixel: fb_info.bytes_per_pixel,
                pixel_format: match fb_info.pixel_format {
                    bootloader_api::info::PixelFormat::Rgb => EarlyFramebufferFormat::Rgb,
                    bootloader_api::info::PixelFormat::Bgr => EarlyFramebufferFormat::Bgr,
                    _ => panic!("Unknown pixel format!")
                },

                x: 0,
                y: 0
            }
        ).unwrap();

        x86_64::info::SYSTEM_INFO_RSDP_ADDR = info.rsdp_addr.into_option();
    }

    println!("hello from rust before enabling mmu!");
    mmu::enable_mmu();
    early_framebuffer::clear_screen();

    log::debug!("hello from rust after enabling nyaa!");

    // Set up kernel heap
    {
        let kernel_aspace = &mut KERNEL_ADDRESS_SPACE.write();
        kernel_aspace.create(
            KERNEL_HEAP_BASE,
            KERNEL_HEAP_INITIAL_SIZE,
            PagePermission::KERNEL_READ_WRITE,
        );
    }
    log::debug!("after setting up heap");
    log::debug!("heap deez nuts");

    platform::scheduler_pre_init();
    log::debug!("scheduler preinit");
    scheduler::init(platform::get_cpu_count());
    log::debug!("scheduler init");

    platform::bringup_other_cpus();
    log::debug!("bringup");

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

    /*let disp_main_thread = init::load_process(disp_buf, "disp");
    scheduler::register_thread(disp_main_thread.clone());*/

    platform::scheduler_post_init();

    log::debug!("Running...");

    scheduler::force_switch_to(fs_main_thread);
    panic!("We shouldn't get here!");
}

#[naked]
#[no_mangle]
unsafe extern "C" fn ap_entry_trampoline() {
    core::arch::asm!("mov rbx, [rip + __ap_stack_pointers]
        mov rsp, [rbx + rdi * 8]
        jmp ap_entry", options(noreturn));
}

#[no_mangle]
extern "C" fn ap_entry(cpu_number: usize) {
    println!("Hello from an AP! ({})", cpu_number);
    platform::scheduler_post_init();
    x86_64::syscall::setup_syscall();
    init::setup_ap_per_cpu(cpu_number);
    x86_64::gdt::setup_gdt();

    let idle_thread = per_cpu::get().idle_thread.as_ref().unwrap().clone();
    println!("AP going idle...");
    scheduler::force_switch_to(idle_thread);
    panic!("We shouldn't get here.");
}
