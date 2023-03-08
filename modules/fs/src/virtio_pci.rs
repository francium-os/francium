use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_structs, register_bitfields};
use tock_registers::interfaces::Readable;
use tock_registers::interfaces::Writeable;
use tock_registers::interfaces::ReadWriteable;

use process::ipc;
use process::syscalls;
use francium_common::types::{MapType, PagePermission};

pub struct VirtioPciDevice {
	common: &'static mut VirtioPciCommonCfg,
	notify: *mut u8,
	isr_status: *mut u8,
	notify_off_multiplier: usize,
	pub queues: Vec<Virtq>
}

// This looks a bit weird because we didn't make VirtqUsed/VirtqAvail DSTs.
//#[derive(Debug)]
pub struct Virtq {
	size: usize,
	index: u16,

	desc: &'static mut [VirtqDesc],
	used: &'static mut VirtqUsed,
	used_ring: &'static mut [VirtqUsedElem],
	avail: &'static mut VirtqAvail,
	avail_ring: &'static mut [u16],

	notify_ptr: *mut u16,

	desc_phys: usize,
	used_phys: usize,
	avail_phys: usize,

	desc_index: usize,
	avail_index: usize
}

impl Virtq {
	fn new(queue_index: u16, queue_size: usize, notify_ptr: *mut u16) -> Virtq {
		assert!(queue_size <= (4096 / 16));

		// Thankfully, QEMU?'s default virtq size is 256.
		// This lets us fit the desc into one page.
		// Used/avail could be packed together.
		
		let desc_virt = syscalls::map_memory(0, 4096, PagePermission::USER_READ_WRITE).unwrap();
		let used_virt = syscalls::map_memory(0, 4096, PagePermission::USER_READ_WRITE).unwrap();
		let avail_virt = syscalls::map_memory(0, 4096, PagePermission::USER_READ_WRITE).unwrap();

		let desc_phys = syscalls::query_physical_address(desc_virt).unwrap();
		let used_phys = syscalls::query_physical_address(used_virt).unwrap();
		let avail_phys = syscalls::query_physical_address(avail_virt).unwrap();

		let q = unsafe {
			Virtq {
				size: queue_size,
				index: queue_index,

				desc: std::slice::from_raw_parts_mut(desc_virt as *mut u8 as *mut VirtqDesc, queue_size),
				used: (used_virt as *mut u8 as *mut VirtqUsed).as_mut().unwrap(),
				used_ring: std::slice::from_raw_parts_mut((used_virt as *mut u8).add(std::mem::size_of::<VirtqUsed>()) as *mut VirtqUsedElem, queue_size),
				avail: (avail_virt as *mut u8 as *mut VirtqAvail).as_mut().unwrap(),
				avail_ring: std::slice::from_raw_parts_mut((avail_virt as *mut u8).add(std::mem::size_of::<VirtqAvail>()) as *mut u16, queue_size),
				
				notify_ptr: notify_ptr,

				desc_phys: desc_phys,
				used_phys: used_phys,
				avail_phys: avail_phys,
				desc_index: 0,
				avail_index: 0
			}
		};

		q.used.flags = 0;
		q.avail.flags = 0;
		q.used.idx = 0;
		q.avail.idx = 0;

		q
	}

	pub fn push_desc(&mut self, desc: VirtqDesc) -> usize {
		let i = self.desc_index;
		self.desc_index += 1;

		self.desc[i] = desc;
		i
	}

	pub fn push_desc_chain(&mut self, desc_chain: &[VirtqDesc]) -> u16 {
		let i = self.desc_index as u16;

		for desc in &desc_chain[..desc_chain.len() - 1] {
			let mut desc_tmp = *desc;
			desc_tmp.flags |= VirtqDesc::F_NEXT;
			desc_tmp.next = self.desc_index as u16 + 1;

			self.desc[self.desc_index] = desc_tmp;
			self.desc_index += 1;
		}
		// Handle last descriptor.
		self.desc[self.desc_index] = desc_chain[desc_chain.len()-1];
		self.desc_index += 1;

		i
	}

	pub fn push_avail(&mut self, idx: u16) {
		self.avail_ring[self.avail_index] = idx;
		self.avail_index += 1;
		self.avail.idx = self.avail_index as u16;
	}

	/*pub fn pop_used(&mut self) -> u16 {
		self.used_ring[self.avail_index] = idx;
		self.avail_index += 1;
		self.avail.idx = self.avail_index as u16;
	}*/

	pub fn notify(&self) {
		unsafe {
			self.notify_ptr.write_volatile(self.index as u16)
		}
	}
}

impl VirtioPciDevice {
	pub fn new(device_id: u32) -> VirtioPciDevice {
		let mut notify_off_multiplier: Option<u32> = None;
		let mut common_info: Option<(u8,u32,u32)> = None;
		let mut isr_status_info: Option<(u8,u32,u32)> = None;
		let mut notify_info: Option<(u8,u32,u32)> = None;
		let mut device_specific_info: Option<(u8,u32,u32)> = None;

		ipc::pcie::enable(device_id).unwrap();

		let mut i = 0;
		while let Ok(cap_data) = ipc::pcie::get_cap(device_id, i) {
			//println!("cap: {:?}", cap);
			// cap type == vendor
			if cap_data[0] == 9 {
				if cap_data.len() >= 16 {
					let pci_cap = VirtioPciCap {
						// [0] pci cap type: u8
						// [1] pci cap next: u8
						// [2] pci vendor specific length: u8
						cfg_type: cap_data[3],
						bar: cap_data[4],
						// 3 bytes padding
						offset: u32::from_le_bytes(cap_data[8..12].try_into().unwrap()),
						length: u32::from_le_bytes(cap_data[12..16].try_into().unwrap())
					};
					println!("pci_cap: {:?}", pci_cap);

					match pci_cap.cfg_type {
						1 => { // VIRTIO_PCI_CAP_COMMON_CFG
							common_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
						},
						2 => { // VIRTIO_PCI_CAP_NOTIFY_CFG has extra data!
							// le32 notify_off_multiplier; /* Multiplier for queue_notify_off. *
							notify_off_multiplier = Some(u32::from_le_bytes(cap_data[16..20].try_into().unwrap()));
							notify_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
						},
						3 => { // VIRTIO_PCI_CAP_ISR_CFG
							isr_status_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
						},
						4 => { // VIRTIO_PCI_CAP_DEVICE_CFG
							device_specific_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
						},
						5 => {}, // Do nothing for VIRTIO_PCI_CAP_PCI_CFG.
						_ => panic!("Invalid VirtIO type!")
					}
				}
			} else if cap_data[0] == 0x11 {
				println!("MSI-X status: {:x?}", u16::from_le_bytes(cap_data[2..4].try_into().unwrap()));
			} else if cap_data[0] == 5 {
				println!("MSI status: {:x?}", u16::from_le_bytes(cap_data[2..4].try_into().unwrap()));
			}
			i += 1;
		}

		println!("{:?} {:?} {:?} {:?}", common_info, notify_info, isr_status_info, device_specific_info);

		let common_info = common_info.unwrap();
		let notify_info = notify_info.unwrap();
		let isr_status_info = isr_status_info.unwrap();
		//let device_specific_info = device_specific_info.unwrap();

		// Assert: all devices share a BAR. At least QEMU does this.
		assert!(common_info.0 == notify_info.0 && notify_info.0 == isr_status_info.0);
		let bar_index = common_info.0;
		let (bar_offset, bar_size) = ipc::pcie::get_bar(device_id, bar_index as u8).unwrap();

		let bar_virt = syscalls::map_device_memory(
            bar_offset,
            0,
            bar_size,
            // Should it be cachable? Probably not.
            MapType::NormalUncachable,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap();

        let common_virt = bar_virt + common_info.1 as usize;
        let notify_virt = bar_virt + notify_info.1 as usize;
        let isr_status_virt = bar_virt + isr_status_info.1 as usize;
        //let device_specific_virt = bar_virt + device_specific_info.1;

		let mut device = VirtioPciDevice {
			common: unsafe { (common_virt as *mut VirtioPciCommonCfg).as_mut().unwrap() },
			notify: notify_virt as *mut u8,
			isr_status: isr_status_virt as *mut u8,
			notify_off_multiplier: notify_off_multiplier.unwrap() as usize,
			queues: Vec::new()
		};

		device.init();
		device
	}

	fn init(&mut self) {
		const VIRTIO_F_VERSION_1: usize = 1<<32;

		/*
		The driver MUST follow this sequence to initialize a device:
		1. Reset the device.
		2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
		3. Set the DRIVER status bit: the guest OS knows how to drive the device.
		4. Read device feature bits, and write the subset of feature bits understood by the OS and driver to the
		device. During this step the driver MAY read (but MUST NOT write) the device-specific configuration
		fields to check that it can support the device before accepting it.
		5. Set the FEATURES_OK status bit. The driver MUST NOT accept new feature bits after this step.
		6. Re-read device status to ensure the FEATURES_OK bit is still set: otherwise, the device does not
		support our subset of features and the device is unusable.
		7. Perform device-specific setup, including discovery of virtqueues for the device, optional per-bus setup,
		reading and possibly writing the device’s virtio configuration space, and population of virtqueues.
		8. Set the DRIVER_OK status bit. At this point the device is “live”
		*/

		self.common.device_status.modify(DeviceStatus::ACKNOWLEDGE::SET);
		self.common.device_status.modify(DeviceStatus::DRIVER::SET);

		// Read feature bits to confirm this is a v1 device.
		self.common.device_feature_select.set(1);
		let features_hi = self.common.device_feature.get();
		if !((features_hi & 1) == 1) {
			panic!("Legacy only device!");
		}

		// Set VIRTIO_F_VERSION_1.
		self.common.driver_feature_select.set(1);
		self.common.driver_feature_select.set(1);

		self.common.device_status.modify(DeviceStatus::FEATURES_OK::SET);

		assert!(self.common.device_status.is_set(DeviceStatus::FEATURES_OK));
		// Features still OK.
		self.discover_queues();
		self.common.device_status.modify(DeviceStatus::DRIVER_OK::SET);
	}

	fn discover_queues(&mut self) {
		let mut i = 0;
		loop {
			self.common.queue_select.set(i);
			let queue_size = self.common.queue_size.get();
			
			if queue_size == 0 {
				break
			}

			// Queue layout:
			// descriptor table (align 16) 16 * queue size
			// available ring   (align 2) 6 + 2*queue size
			// used ring        (align 4) 4 + 8*queue size

			// • Descriptor Table - for the Descriptor Area
			// • Available Ring - for the Driver Area
			// • Used Ring - for the Device Area

			let queue_notify_off: usize = self.common.queue_notify_off.get() as usize;
			let queue_notify_ptr = unsafe {
				self.notify.add(queue_notify_off * self.notify_off_multiplier) as *mut u16
			};

			let q = Virtq::new(i as u16, queue_size as usize, queue_notify_ptr);

			self.common.queue_desc.set(q.desc_phys as u64);
			self.common.queue_driver.set(q.avail_phys as u64);
			self.common.queue_device.set(q.used_phys as u64);

			self.common.queue_enable.set(1);

			self.queues.push(q);

			i += 1;
		}
	}
}

#[derive(Debug)]
struct VirtioPciCap {
	cfg_type: u8,
	bar: u8,
	offset: u32,
	length: u32
}

/*
ACKNOWLEDGE (1) Indicates that the guest OS has found the device and recognized it as a valid virtio
device.
DRIVER (2) Indicates that the guest OS knows how to drive the device.
Note: There could be a significant (or infinite) delay before setting this bit. For example, under Linux,
drivers can be loadable modules.
FAILED (128) Indicates that something went wrong in the guest, and it has given up on the device. This
could be an internal error, or the driver didn’t like the device for some reason, or even a fatal error
during device operation.
FEATURES_OK (8) Indicates that the driver has acknowledged all the features it understands, and feature
negotiation is complete.
DRIVER_OK (4) Indicates that the driver is set up and ready to drive the device.
DEVICE_NEEDS_RESET (64) Indicates that the device has experienced an error from which it can’t recover
*/

register_bitfields! [
    u8,
    DeviceStatus [
    	FAILED             7, // 128
    	DEVICE_NEEDS_RESET 6, // 64
    	FEATURES_OK        3, // 8
    	DRIVER_OK          2, // 4
		DRIVER             1, // 2
        ACKNOWLEDGE        0  // 1
    ]
];

/*
From spec:

struct virtio_pci_common_cfg {
/* About the whole device. */
le32 device_feature_select; /* read-write */
le32 device_feature; /* read-only for driver */
le32 driver_feature_select; /* read-write */
le32 driver_feature; /* read-write */
le16 msix_config; /* read-write */
le16 num_queues; /* read-only for driver */
u8 device_status; /* read-write */
u8 config_generation; /* read-only for driver */
/* About a specific virtqueue. */
le16 queue_select; /* read-write */
le16 queue_size; /* read-write */
le16 queue_msix_vector; /* read-write */
le16 queue_enable; /* read-write */
le16 queue_notify_off; /* read-only for driver */
le64 queue_desc; /* read-write */
le64 queue_driver; /* read-write */
le64 queue_device; /* read-write */
};
*/

register_structs! {
    VirtioPciCommonCfg {
        (0x00 => device_feature_select: ReadWrite<u32>),
        (0x04 => device_feature: ReadOnly<u32>),
		(0x08 => driver_feature_select: ReadWrite<u32>),
		(0x0c => driver_feature: ReadWrite<u32>),
		(0x10 => msix_config: ReadWrite<u16>),
		(0x12 => num_queues: ReadOnly<u16>),
		(0x14 => device_status: ReadWrite<u8, DeviceStatus::Register>),
		(0x15 => config_generation: ReadOnly<u8>),

		(0x16 => queue_select: ReadWrite<u16>),
		(0x18 => queue_size: ReadWrite<u16>),
		(0x1a => queue_msix_vector: ReadWrite<u16>),
		(0x1c => queue_enable: ReadWrite<u16>),
		(0x1e => queue_notify_off: ReadOnly<u16>),
		(0x20 => queue_desc: ReadWrite<u64>),
		(0x28 => queue_driver: ReadWrite<u64>),
		(0x30 => queue_device: ReadWrite<u64>),

        // The end of the struct is marked as follows.
        (0x38 => @END),
    }
}


/*
struct virtq_desc {
/* Address (guest-physical). */
le64 addr;
/* Length. */
le32 len;
/* This marks a buffer as continuing via the next field. */
#define VIRTQ_DESC_F_NEXT 1
/* This marks a buffer as device write-only (otherwise device read-only). */
#define VIRTQ_DESC_F_WRITE 2
/* This means the buffer contains a list of buffer descriptors. */
#define VIRTQ_DESC_F_INDIRECT 4
/* The flags as indicated above. */
le16 flags;
/* Next field if flags & NEXT */
le16 next;
};*/

/*
struct virtq_avail {
#define VIRTQ_AVAIL_F_NO_INTERRUPT 1
le16 flags;
le16 idx;
le16 ring[ /* Queue Size */ ];
le16 used_event; /* Only if VIRTIO_F_EVENT_IDX */
};
*/

/*
struct virtq_used {
#define VIRTQ_USED_F_NO_NOTIFY 1
le16 flags;
le16 idx;
struct virtq_used_elem ring[ /* Queue Size */];
le16 avail_event; /* Only if VIRTIO_F_EVENT_IDX */
};
/* le32 is used here for ids for padding reasons. */
struct virtq_used_elem {
/* Index of start of used descriptor chain. */
le32 id;
/* Total length of the descriptor chain which was used (written to) */
le32 len;
};
*/

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VirtqDesc {
	pub addr: u64, /* Address (guest-physical). */
	pub len: u32,

	/* The flags as indicated above. */
	pub flags: u16,
	/* Next field if flags & NEXT */
	pub next: u16
}

impl VirtqDesc {
	/* This marks a buffer as continuing via the next field. */
	pub const F_NEXT: u16 = 1;
	/* This marks a buffer as device write-only (otherwise device read-only). */
	pub const F_WRITE: u16 = 2;
	/* This means the buffer contains a list of buffer descriptors. */
	pub const F_INDIRECT: u16 = 4;

	pub fn new(addr: u64, len: u32, flags: u16) -> VirtqDesc {
		VirtqDesc {
			addr: addr,
			len: len,
			flags: flags,
			next: 0
		}
	}
}

const VIRTQ_AVAIL_F_NO_INTERRUPT: u16 = 1;
#[repr(C)]
struct VirtqAvail {
	flags: u16,
	idx: u16,
	//ring: [u16],
	//used_event: u16 /* Only if VIRTIO_F_EVENT_IDX */
}

const VIRTQ_USED_F_NO_NOTIFY: u16 = 1;
#[repr(C)]
struct VirtqUsed {
	flags: u16,
	idx: u16,
	//ring: [VirtqUsedElem],
	//avail_event: u16 /* Only if VIRTIO_F_EVENT_IDX */
}

/* le32 is used here for ids for padding reasons. */
#[repr(C)]
struct VirtqUsedElem {
	id: u32, /* Index of start of used descriptor chain. */
	len: u32 /* Total length of the descriptor chain which was used (written to) */
}