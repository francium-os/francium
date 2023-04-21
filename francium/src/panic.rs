use crate::println;
use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::debug!("kernel {}", info);
    println!("kernel {}", info);
    loop {}
}
