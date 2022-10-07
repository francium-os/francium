pub const MAX_TRANSLATE: usize = 4;
pub const TRANSLATE_TYPE_MOVE_HANDLE: u64 = 1;
pub const TRANSLATE_TYPE_COPY_HANDLE: u64 = 2;

pub struct IPCHeader {
	pub id: u32,
	pub size: usize,
	pub translate_count: usize
}

impl IPCHeader {
	pub fn pack(header: &IPCHeader) -> u32 {
		assert!(header.size < 256);
		assert!(header.translate_count < 256);

		let packed = header.id | (((header.size & 0xff) as u32) << 8) | (((header.translate_count & 0xff) as u32) << 16);
		packed
	}

	pub fn unpack(packed: u32) -> IPCHeader {
		let message_id = packed & 0xff;
		let message_size = (packed & (0xff<<8))>>8;
		let message_translate_count = (packed & (0xff<<16))>>16;

		IPCHeader{id: message_id, size: message_size as usize, translate_count: message_translate_count as usize }
	}
}