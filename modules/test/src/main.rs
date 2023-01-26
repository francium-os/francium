use process::syscalls;
use process::ipc;

const SECOND: u64 = 1_000_000_000;

fn main() {
    println!("Hello from test!");

    /*println!("FS IPC");
    ipc::fs::stop();
    println!("SM IPC");
    ipc::sm::stop();
    println!("Done");*/

    println!("PCI devices: {:?}", ipc::pcie::list_devices());

    println!("Sleeping for 1 second...");
    syscalls::sleep_ns(1 * SECOND);
    println!("*yawn*");

    loop {
        syscalls::sleep_ns(1 * SECOND);
    }

    //syscalls::exit_process();
}
