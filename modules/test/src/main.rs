//use process::ipc;
use process::syscalls;

const SECOND: u64 = 1_000_000_000;

fn main() {
    println!("Hello from test!");

    println!("Sleeping for 1 second...");
    syscalls::sleep_ns(1 * SECOND);
    println!("*yawn*");

    syscalls::exit_process();
}
