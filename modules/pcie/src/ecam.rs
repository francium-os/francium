/* PCIE header */

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct ECAMHeader {
	pub vendor_id: u16,
	pub device_id: u16,
	
	pub command: u16,
	pub status: u16,
	
	pub revision: u8,

	pub prog_if: u8,
	pub subclass: u8,
	pub class: u8,

	pub cache_line_size: u8,
	pub latency_timer: u8,
	pub header_type: u8,
	pub bist: u8,
}

/*struct ECAMDevice {

}*/

/*struct ECAMBridge {

}*/