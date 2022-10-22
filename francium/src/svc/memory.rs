use crate::{scheduler, PagePermission};
use common::os_error::{ResultCode, RESULT_OK};

//pub const PROT_NONE: u32 = 0x0000;
pub const PROT_EXEC: u32 = 0x0001;
pub const PROT_WRITE: u32 = 0x0002;
pub const PROT_READ: u32 = 0x0004;


pub fn svc_map_memory(address: usize, length: usize, permission: u32) -> (ResultCode, usize) {
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

	let mut page_permission: PagePermission = PagePermission::READ_ONLY;

	if permission & PROT_EXEC == PROT_EXEC {
		page_permission |= PagePermission::EXECUTE;
	}
	
	if permission & PROT_WRITE == PROT_WRITE {
		page_permission |= PagePermission::WRITE;
	}

	if permission & PROT_READ == PROT_READ {
		page_permission |= PagePermission::READ_ONLY;
	}

	aspace.create(highest_mmap, length, page_permission);

	(RESULT_OK, highest_mmap)
}