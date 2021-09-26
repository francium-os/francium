use core::cmp::min;

extern "C" {
	pub fn syscall_debug_output(s: *const u8, len: usize);
	pub fn syscall_create_port(tag: u64);
	pub fn syscall_connect_to_port(tag: u64);
	pub fn syscall_exit_process();
}

pub fn print(s: &str) {
	unsafe {
		syscall_debug_output(s.as_bytes().as_ptr(), s.len());
	}
}

fn make_tag(s: &str) -> u64 {
	let tag_bytes = s.as_bytes();
	let length = min(8, tag_bytes.len());
	let mut tag_bytes_padded: [u8; 8] = [0; 8];
	tag_bytes_padded[0..length].copy_from_slice(tag_bytes);
	
	u64::from_be_bytes(tag_bytes_padded)
}

pub fn create_port(s: &str) {
	unsafe {
		syscall_create_port(make_tag(s));
	}
}

pub fn connect_to_port(s: &str) {
	unsafe {
		syscall_connect_to_port(make_tag(s));
	}
}

pub fn exit_process() {
	unsafe {
		syscall_exit_process();
	}
}