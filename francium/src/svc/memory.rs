use tracing::{event, Level};

use crate::scheduler;
use crate::mmu::PagePermission;
use francium_common::types::PhysAddr;
use common::os_error::{ResultCode, RESULT_OK};

pub fn svc_map_memory(address: usize, length: usize, permission: u64) -> (ResultCode, usize) {
    event!(
        Level::TRACE,
        svc_name = "map_memory",
        address = address,
        length = length,
        permission = permission
    );

    let binding = scheduler::get_current_process();
    let mut process_locked = binding.lock();
    let aspace = &mut process_locked.address_space;

    if address != 0 {
        panic!("Don't know how to deal with mmap hints");
    }

    const MMAP_BASE: usize = 0x100000000;
    let mut highest_mmap: usize = MMAP_BASE;
    for region in &aspace.regions {
        if region.address + region.size >= highest_mmap {
            highest_mmap = region.address + region.size;
        }
    }

    let page_permission: PagePermission = PagePermission::from_bits(permission).unwrap();
    aspace.create(highest_mmap, length, page_permission);
    //println!("{:x?}", aspace.regions);

    (RESULT_OK, highest_mmap)
}

pub fn svc_map_device_memory(phys_address: PhysAddr, virt_address: usize, length: usize, permission: u64) -> (ResultCode, usize) {
    event!(
        Level::TRACE,
        svc_name = "map_device_memory",
        phys_address = phys_address.0,
        virt_address = virt_address,
        length = length,
        permission = permission
    );

    let binding = scheduler::get_current_process();
    let mut process_locked = binding.lock();
    let aspace = &mut process_locked.address_space;

    if virt_address != 0 {
        panic!("Don't know how to deal with device memory mmap hints");
    }

    const MMAP_BASE: usize = 0x100000000;
    let mut highest_mmap: usize = MMAP_BASE;
    for region in &aspace.regions {
        if region.address + region.size >= highest_mmap {
            highest_mmap = region.address + region.size;
        }
    }

    let page_permission: PagePermission = PagePermission::from_bits(permission).unwrap();
    aspace.alias(phys_address, highest_mmap, length, page_permission);

    (RESULT_OK, highest_mmap)
}
