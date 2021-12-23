use crate::syscalls;

extern "C" {
    fn main(argc: isize, argv: *const *const u8) -> isize;
}

#[no_mangle]
extern "C" fn _start() -> ! {
    let argv = [];
    unsafe { main(0, argv.as_ptr()) };
    syscalls::exit_process()
}

trait Termination {}

impl Termination for () {}

#[lang = "start"]
fn lang_start<T: Termination>(main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize {
    main();
    syscalls::exit_process();
}

use core::panic::PanicInfo;
/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // printing info directly seems to just die (tm)
    // idk why, but not doing that seems to work better
    println!("user mode panic: {:?}", info.message());
    syscalls::exit_process()
}