extern "C" {
	pub fn syscall_debug_output(s: *const u8, len: usize);
}

pub fn print(s: &str) {
	unsafe {
		syscall_debug_output(s.as_bytes().as_ptr(), s.len());
	}
}