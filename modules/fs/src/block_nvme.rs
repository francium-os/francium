use crate::block::BlockDevice;

struct BlockNVME {

}

impl BlockDevice for BlockNVME {
	// todo
	fn read(&self, offset: usize, buffer: &mut [u8]) -> usize {
		0
	}

	fn write(&self, offset: usize, buffer: &[u8]) -> usize {
		0
	}
}

pub fn scan() -> Vec<Box<dyn BlockDevice>> {
	Vec::new()
}