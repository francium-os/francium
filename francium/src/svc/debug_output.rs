pub fn svc_debug_output(user_ptr: *const u8, len: usize) {
	let mut temp_buffer: [u8; 512] = [0; 512];
	unsafe {
		core::ptr::copy_nonoverlapping(user_ptr, temp_buffer.as_mut_ptr(), len);
	}

	print!("{}", core::str::from_utf8(&temp_buffer[0..len]).unwrap());
}