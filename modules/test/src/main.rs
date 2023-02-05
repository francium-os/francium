use process::syscalls;
use process::ipc;

const SECOND: u64 = 1_000_000_000;

mod bochs;

fn main() {
    println!("Hello from test!");

    /*println!("FS IPC");
    ipc::fs::stop();
    println!("SM IPC");
    ipc::sm::stop();
    println!("Done");*/

    println!("PCI devices: {:x?}", ipc::pcie::list_devices());

    println!("Sleeping for 1 second...");
    syscalls::sleep_ns(1 * SECOND);
    println!("*yawn*");

    /*let virt = syscalls::map_device_memory(0xc0000000, 0, align_up(1280*3*720, 0x1000), PagePermission::USER_READ_WRITE).unwrap();
    unsafe {
        core::ptr::write_bytes(virt as *mut u8, 0xaa, 1280*3*720);
    }

    for i in 0..=255 {
        syscalls::sleep_ns(SECOND/60);
        unsafe {
            core::ptr::write_bytes(virt as *mut u8, i, 1280*3*720);
        }
    }*/
    // TODO: PCI sysmodule should export BARs as sharedmem to apps.

    let mut bochs = bochs::BochsAdapter::new().unwrap();
    bochs.set_mode(640, 480);
    bochs.fill();

    syscalls::exit_process();
}
