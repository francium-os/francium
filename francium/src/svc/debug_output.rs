pub fn svc_debug_output(user_ptr: *const u8, len: usize) {
    let mut temp_buffer: [u8; 1024] = [0; 1024];
    unsafe {
        core::ptr::copy_nonoverlapping(user_ptr, temp_buffer.as_mut_ptr(), len);
    }

    // These are on seperate lines so a potential panic doesn't occur inside the print
    // (which locks the serial port, leading to a hang when panic tries to print).
    let as_utf8 = core::str::from_utf8(&temp_buffer[0..len]).unwrap();

    // Strip a newline off the end, if it's present. Log will add one for us.
    if &as_utf8[len - 1..len] == "\n" {
        log::debug!("{}", &as_utf8[0..len - 1]);
    } else {
        log::debug!("{}", as_utf8);
    }
}
