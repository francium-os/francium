use tock_registers::interfaces::ReadWriteable;
use tock_registers::interfaces::Readable;
use tock_registers::interfaces::Writeable;
use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{register_bitfields, register_structs};

use francium_common::types::{MapType, PagePermission};
use process::ipc;
use process::syscalls;
use process::Handle;

pub struct VirtioNotifier {
    isr_status: *mut u8,
    interrupt_event: Handle,
}

// safety: dude trust me
unsafe impl Send for VirtioPciDevice {}
pub struct VirtioPciDevice {
    common: &'static mut VirtioPciCommonCfg,
    notify: *mut u8,
    notify_off_multiplier: usize,
    pub device_specific: *mut u8,
    pub queues: Vec<Virtq>,
    pub legacy_notifier: VirtioNotifier,
}

// This looks a bit weird because we didn't make VirtqUsed/VirtqAvail DSTs.
//#[derive(Debug)]
pub struct Virtq {
    size: usize,
    index: u16,

    desc: &'static mut [VirtqDesc],
    used: &'static mut VirtqUsed,
    _used_ring: &'static mut [VirtqUsedElem],
    avail: &'static mut VirtqAvail,
    avail_ring: &'static mut [u16],

    notify_ptr: *mut u16,

    desc_phys: usize,
    used_phys: usize,
    avail_phys: usize,

    desc_index: usize,
    avail_index: usize,
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

                desc: std::slice::from_raw_parts_mut(
                    desc_virt as *mut u8 as *mut VirtqDesc,
                    queue_size,
                ),
                used: (used_virt as *mut u8 as *mut VirtqUsed).as_mut().unwrap(),
                _used_ring: std::slice::from_raw_parts_mut(
                    (used_virt as *mut u8).add(std::mem::size_of::<VirtqUsed>())
                        as *mut VirtqUsedElem,
                    queue_size,
                ),
                avail: (avail_virt as *mut u8 as *mut VirtqAvail).as_mut().unwrap(),
                avail_ring: std::slice::from_raw_parts_mut(
                    (avail_virt as *mut u8).add(std::mem::size_of::<VirtqAvail>()) as *mut u16,
                    queue_size,
                ),

                notify_ptr: notify_ptr,

                desc_phys: desc_phys,
                used_phys: used_phys,
                avail_phys: avail_phys,
                desc_index: 0,
                avail_index: 0,
            }
        };

        q.used.flags = 0;
        q.avail.flags = 0;
        q.used.idx = 0;
        q.avail.idx = 0;

        q
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
        self.desc[self.desc_index] = desc_chain[desc_chain.len() - 1];
        self.desc_index += 1;

        i
    }

    pub fn push_avail(&mut self, idx: u16) {
        self.avail_ring[self.avail_index] = idx;
        self.avail_index += 1;

        if self.avail_index == self.size {
            self.avail_index = 0;
        }

        self.avail.idx = self.avail_index as u16;
    }

    /*pub fn pop_used(&mut self) -> u16 {
        self.used_ring[self.avail_index] = idx;
        self.avail_index += 1;
        self.avail.idx = self.avail_index as u16;
    }*/

    pub fn notify(&self) {
        unsafe { self.notify_ptr.write_volatile(self.index as u16) }
    }
}

impl VirtioPciDevice {
    pub fn new(device_id: u32) -> VirtioPciDevice {
        let mut notify_off_multiplier: Option<u32> = None;
        let mut common_info: Option<(u8, u32, u32)> = None;
        let mut isr_status_info: Option<(u8, u32, u32)> = None;
        let mut notify_info: Option<(u8, u32, u32)> = None;
        let mut device_specific_info: Option<(u8, u32, u32)> = None;

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
                        length: u32::from_le_bytes(cap_data[12..16].try_into().unwrap()),
                    };
                    //println!("pci_cap: {:?}", pci_cap);

                    match pci_cap.cfg_type {
                        1 => {
                            // VIRTIO_PCI_CAP_COMMON_CFG
                            common_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
                        }
                        2 => {
                            // VIRTIO_PCI_CAP_NOTIFY_CFG has extra data!
                            // le32 notify_off_multiplier; /* Multiplier for queue_notify_off. *
                            notify_off_multiplier =
                                Some(u32::from_le_bytes(cap_data[16..20].try_into().unwrap()));
                            notify_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
                        }
                        3 => {
                            // VIRTIO_PCI_CAP_ISR_CFG
                            isr_status_info = Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
                        }
                        4 => {
                            // VIRTIO_PCI_CAP_DEVICE_CFG
                            device_specific_info =
                                Some((pci_cap.bar, pci_cap.offset, pci_cap.length));
                        }
                        5 => {} // Do nothing for VIRTIO_PCI_CAP_PCI_CFG.
                        _ => panic!("Invalid VirtIO type!"),
                    }
                }
            } else if cap_data[0] == 0x11 {
                /*println!(
                    "MSI-X status: {:x?}",
                    u16::from_le_bytes(cap_data[2..4].try_into().unwrap())
                );*/
            } else if cap_data[0] == 5 {
                /*println!(
                    "MSI status: {:x?}",
                    u16::from_le_bytes(cap_data[2..4].try_into().unwrap())
                );*/
            }
            i += 1;
        }

        let common_info = common_info.unwrap();
        let notify_info = notify_info.unwrap();
        let isr_status_info = isr_status_info.unwrap();
        let device_specific_info = device_specific_info.unwrap();

        // Assert: all devices share a BAR. At least QEMU does this.
        assert!(common_info.0 == notify_info.0 && notify_info.0 == isr_status_info.0);
        let bar_index = common_info.0;
        let (bar_offset, bar_size) = ipc::pcie::get_bar(device_id, bar_index as u8).unwrap();

        let bar_virt = syscalls::map_device_memory(
            bar_offset,
            0,
            bar_size,
            // Should it be cachable? Probably not.
            MapType::Device,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap();

        let common_virt = bar_virt + common_info.1 as usize;
        let notify_virt = bar_virt + notify_info.1 as usize;
        let isr_status_virt = bar_virt + isr_status_info.1 as usize;
        let device_specific_virt = bar_virt + device_specific_info.1 as usize;

        let interrupt = ipc::pcie::get_interrupt_event(device_id).unwrap().0;

        let mut device = VirtioPciDevice {
            common: unsafe { (common_virt as *mut VirtioPciCommonCfg).as_mut().unwrap() },
            notify: notify_virt as *mut u8,
            notify_off_multiplier: notify_off_multiplier.unwrap() as usize,
            device_specific: device_specific_virt as *mut u8,
            queues: Vec::new(),
            legacy_notifier: VirtioNotifier {
                isr_status: isr_status_virt as *mut u8,
                interrupt_event: interrupt,
            },
        };

        device.init();
        device
    }

    fn init(&mut self) {
        const VIRTIO_F_VERSION_1: u64 = 1 << 32;

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

        self.common
            .device_status
            .modify(DeviceStatus::ACKNOWLEDGE::SET);
        self.common.device_status.modify(DeviceStatus::DRIVER::SET);

        // Read feature bits to confirm this is a v1 device.
        self.common.device_feature_select.set(1);
        let features_hi = self.common.device_feature.get();

        if !((features_hi & 1) == 1) {
            panic!("Legacy only device!");
        }

        // Set VIRTIO_F_VERSION_1.
        self.common.driver_feature_select.set(1);
        self.common
            .driver_feature
            .set((VIRTIO_F_VERSION_1 << 32) as u32);

        self.common
            .device_status
            .modify(DeviceStatus::FEATURES_OK::SET);

        assert!(self.common.device_status.is_set(DeviceStatus::FEATURES_OK));
        // Features still OK.
        self.discover_queues();
        self.common
            .device_status
            .modify(DeviceStatus::DRIVER_OK::SET);
    }

    fn discover_queues(&mut self) {
        let mut i = 0;
        loop {
            self.common.queue_select.set(i);
            let queue_size = self.common.queue_size.get();

            if queue_size == 0 {
                break;
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
                self.notify
                    .add(queue_notify_off * self.notify_off_multiplier) as *mut u16
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

impl VirtioNotifier {
    pub fn wait_for_isr(&self) -> u8 {
        syscalls::wait_one(self.interrupt_event).unwrap();
        let isr_status = unsafe { self.isr_status.read_volatile() };
        syscalls::clear_event(self.interrupt_event).unwrap();

        isr_status
    }
}

#[derive(Debug)]
struct VirtioPciCap {
    cfg_type: u8,
    bar: u8,
    offset: u32,
    length: u32,
}

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VirtqDesc {
    pub addr: u64, /* Address (guest-physical). */
    pub len: u32,

    /* The flags as indicated above. */
    pub flags: u16,
    /* Next field if flags & NEXT */
    pub next: u16,
}

impl VirtqDesc {
    /* This marks a buffer as continuing via the next field. */
    pub const F_NEXT: u16 = 1;
    /* This marks a buffer as device write-only (otherwise device read-only). */
    pub const F_WRITE: u16 = 2;
    /* This means the buffer contains a list of buffer descriptors. */
    //pub const F_INDIRECT: u16 = 4;

    pub fn new(addr: u64, len: u32, flags: u16) -> VirtqDesc {
        VirtqDesc {
            addr: addr,
            len: len,
            flags: flags,
            next: 0,
        }
    }
}

// const VIRTQ_AVAIL_F_NO_INTERRUPT: u16 = 1;
#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    //ring: [u16],
    //used_event: u16 /* Only if VIRTIO_F_EVENT_IDX */
}

// const VIRTQ_USED_F_NO_NOTIFY: u16 = 1;
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
    id: u32,  /* Index of start of used descriptor chain. */
    len: u32, /* Total length of the descriptor chain which was used (written to) */
}
