use process::ipc;
use process::syscalls;

const SECOND: u64 = 1_000_000_000;

fn main() {
    println!("Hello from test!");

    if let Ok(file_handle) = ipc::fs::open_file("efi/boot/bootx64.efi".to_string()) {
        println!("Hello again from test: {:?}", file_handle);
        println!("Reading file: {:?}", ipc::fs::read_file(file_handle.0, 0));
    } else {
        println!("Probably failed to open file..");
    }

    println!("Sleeping for 1 second...");
    syscalls::sleep_ns(1 * SECOND);
    println!("*yawn*");

    syscalls::exit_process();
}
