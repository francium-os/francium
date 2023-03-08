use crate::block::BlockDevice;
use crate::virtio_pci::VirtioPciDevice;
use process::ipc;
use process::syscalls;
use francium_common::types::{MapType, PagePermission};
use crate::virtio_pci::VirtqDesc;

struct BlockVirtio {

}

impl BlockDevice for BlockVirtio {
	// todo
	fn read(&self, offset: usize, buffer: &mut [u8]) -> usize {
		0
	}

	fn write(&self, offset: usize, buffer: &[u8]) -> usize {
		0
	}
}

/*
struct virtio_blk_req {
le32 type;
le32 reserved;
le64 sector;
u8 data[];
u8 status;
};
*/

struct VirtioBlkReq {
	ty: u32,
	_reserved: u32,
	sector: u64,
	// data
}

pub fn scan() -> Vec<Box<dyn BlockDevice>> {
	// block device transitional id is 0x1001
	let transitional_devices = ipc::pcie::get_devices_by_vidpid(0x1af4, 0x1001);
	// new device id 2, +0x1040
	let new_devices = ipc::pcie::get_devices_by_vidpid(0x1af4, 0x1042);

	println!("devices: {:?}", transitional_devices);

	for dev in transitional_devices {
		let mut virtio_dev = VirtioPciDevice::new(dev);

		let buffer_virt = syscalls::map_memory(0, 4096, PagePermission::USER_READ_WRITE).unwrap();
		let buffer_phys = syscalls::query_physical_address(buffer_virt).unwrap();

		let mut q = virtio_dev.queues.get_mut(0).unwrap();

		let request_buffer = q.push_desc_chain(&[
			VirtqDesc::new(buffer_phys as u64, 16, 0),
			VirtqDesc::new(buffer_phys as u64 + 16, 513, VirtqDesc::F_WRITE)
		]);
		
		unsafe {
			(buffer_virt as *mut u32).write_volatile(0);
			(buffer_virt as *mut u32).add(1).write_volatile(0);
			(buffer_virt as *mut u32).add(2).write_volatile(0);
			(buffer_virt as *mut u32).add(3).write_volatile(0);
		}

		// Add!
		q.push_avail(request_buffer);
		q.notify();

		// Wait for IRQ...

		println!("{:x?}", unsafe { &std::slice::from_raw_parts(buffer_virt as *mut u8, 512 + 16)[16..16+512]});

		unsafe { (buffer_virt as *mut u32).add(2).write_volatile(1); }
		println!("Two!");
		q.push_avail(request_buffer);
		q.notify();

		//println!("status: {}", unsafe { q.isr_status.read_volatile() });
		syscalls::sleep_ns(1000000000);
		println!("{:x?}", unsafe { &std::slice::from_raw_parts(buffer_virt as *mut u8, 512 + 16)[16..16+512]});
	}

	/*for dev in new_devices {
		let virtio_dev = VirtioPciDevice::new(dev);
	}*/

	Vec::new()
}