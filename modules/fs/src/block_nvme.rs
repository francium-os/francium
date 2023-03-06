use crate::block::BlockDevice;
use process::ipc;

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
	let nvme_devices = ipc::pcie::get_devices_by_class(1, 8);
	println!("NVMe devices: {:?}", nvme_devices);
	Vec::new()
}