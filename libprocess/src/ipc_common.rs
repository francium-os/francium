#[thread_local]
#[no_mangle]
pub static mut IPC_BUFFER: [u32; 32] = [0; 32];