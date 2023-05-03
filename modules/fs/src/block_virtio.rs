use crate::block::BlockDevice;
use crate::virtio_pci::{VirtioPciDevice, VirtqDesc};
use francium_common::types::PagePermission;
use process::ipc;
use process::syscalls;

struct BlockVirtio {
    virtio_dev: VirtioPciDevice,

    _request_phys: usize,
    request_virt: usize,

    request_buffer_offset: u16,
}

impl BlockVirtio {
    fn new(mut virtio_dev: VirtioPciDevice) -> BlockVirtio {
        let request_virt = syscalls::map_memory(0, 4096, PagePermission::USER_READ_WRITE).unwrap();
        let request_phys = syscalls::query_physical_address(request_virt).unwrap();

        let q = virtio_dev.queues.get_mut(0).unwrap();

        let request_buffer = q.push_desc_chain(&[
            VirtqDesc::new(request_phys as u64, 16, 0),
            VirtqDesc::new(request_phys as u64 + 16, 513, VirtqDesc::F_WRITE),
        ]);

        /*
        unsafe {
            (request_virt as *mut u32).add(2).write_volatile(1);
        }
        */

        BlockVirtio {
            virtio_dev: virtio_dev,
            _request_phys: request_phys,
            request_virt: request_virt,
            request_buffer_offset: request_buffer,
        }
    }
}

impl BlockDevice for BlockVirtio {
    fn read_sector(&mut self, offset: u64, buffer: &mut [u8]) -> u64 {
        let q = self.virtio_dev.queues.get_mut(0).unwrap();
        let notifier = &self.virtio_dev.legacy_notifier;

        //println!("Read: {:x}", offset);

        unsafe {
            (self.request_virt as *mut u32).write_volatile(0); // Type
            (self.request_virt as *mut u32).add(1).write_volatile(0); // Reserved
            (self.request_virt as *mut u32)
                .add(2)
                .write_volatile(offset as u32); // Sector offset (u64)
            (self.request_virt as *mut u32).add(3).write_volatile(0);
        }

        q.push_avail(self.request_buffer_offset);
        q.notify();

        notifier.wait_for_isr();

        // TODO: actually look at the used ring to make sure the request completed...
        unsafe {
            std::ptr::copy_nonoverlapping(
                (self.request_virt as *mut u8).add(16),
                buffer.as_mut_ptr(),
                512,
            );
        }

        /*println!("{:x?}", unsafe {
            &std::slice::from_raw_parts(self.request_virt as *mut u8, 512 + 16)[16..16 + 512]
        });*/

        1
    }

    fn write_sector(&mut self, _offset: u64, _buffer: &[u8]) -> u64 {
        todo!();
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

/*struct VirtioBlkReq {
    ty: u32,
    _reserved: u32,
    sector: u64,
    // data
}*/

pub fn scan() -> Vec<Box<dyn BlockDevice + Send>> {
    // block device transitional id is 0x1001
    let transitional_devices = ipc::pcie::get_devices_by_vidpid(0x1af4, 0x1001);
    // new device id 2, +0x1040
    // let new_devices = ipc::pcie::get_devices_by_vidpid(0x1af4, 0x1042);

    let mut blocks = Vec::new();

    for dev in transitional_devices {
        let virtio_dev = VirtioPciDevice::new(dev);
        let block = BlockVirtio::new(virtio_dev);
        let boxed: Box<dyn BlockDevice + Send> = Box::new(block);

        blocks.push(boxed)
    }

    blocks
}
