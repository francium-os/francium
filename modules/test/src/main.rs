use process::ipc;
use process::syscalls;

const SECOND: u64 = 1_000_000_000;

fn main() {
    println!("Hello from test!");

    let file_handle = ipc::fs::open_file("test.txt").unwrap();
    println!("Hello again from test: {:?}", file_handle);

    println!("Sleeping for 1 second...");
    syscalls::sleep_ns(1 * SECOND);
    println!("*yawn*");

    syscalls::exit_process();
}
