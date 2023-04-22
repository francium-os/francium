use common::system_info::*;
use process::syscalls;

mod ps2;

fn main() {
    println!("Hello from ps2!");
    ps2::scan();
    syscalls::exit_process();
}
