pub fn write_uart(a: &str) {
	let uart_base: *mut u8 = 0x09000000 as *mut u8;
	for c in a.chars() {
		unsafe {
			*uart_base = c as u8;
		}
	}
}

pub fn write_uart_bytes(a: &[u8]) {
	let uart_base: *mut u8 = 0x09000000 as *mut u8;
	for c in a {
		unsafe {
			*uart_base = *c;
		}
	}
}