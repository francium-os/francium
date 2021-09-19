extern "C" {
    fn main(argc: isize, argv: *const *const u8) -> isize;
}

#[no_mangle]
extern "C" fn _start() -> ! {
    let argv = [];
    unsafe { main(0, argv.as_ptr()) };
    unreachable!()
}

trait Termination {}

impl Termination for () {}

#[lang = "start"]
fn lang_start<T: Termination>(main: fn() -> T, argc: isize, argv: *const *const u8) -> isize {
    main();
    2
}

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}