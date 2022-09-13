#[thread_local]
#[no_mangle]
pub static mut IPC_BUFFER: [u8; 128] = [0; 128];