pub fn write_uart(a: &str) {
	let physmap_base: usize = 0xffffff0000000000;

	let uart_base: *mut u8 = (physmap_base + 0x09000000) as *mut u8;
	for c in a.chars() {
		unsafe {
			uart_base.write_volatile(c as u8);
		}
	}
}

pub fn write_uart_bytes(a: &[u8]) {
	let physmap_base: usize = 0xffffff0000000000;

	let uart_base: *mut u8 = (physmap_base + 0x09000000) as *mut u8;
	for c in a {
		unsafe {
			uart_base.write_volatile(*c);
		}
	}
}