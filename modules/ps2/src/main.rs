use common::system_info::*;
use process::syscalls;

#[cfg(target_arch = "x86_64")]
mod ps2;

#[cfg(target_arch = "x86_64")]
fn main() {
    println!("Hello from ps2!");
    ps2::scan();
    syscalls::exit_process();
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    syscalls::exit_process();
}