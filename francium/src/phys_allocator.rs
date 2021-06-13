use crate::mmu::{PhysAddr, phys_to_virt};

// classic 4k pages everywhere
// dumb linked list

static mut PHYS_FREELIST: Option<PhysAddr> = None;

#[derive(Copy, Clone)]
struct PhysEntry {
	next: Option<PhysAddr>
}

unsafe fn read_phys<T: Copy>(addr: PhysAddr) -> T {
	let virt_addr = phys_to_virt(addr);
	*(virt_addr as *const T)
}

unsafe fn write_phys<T>(addr: PhysAddr, value: T) {
	let virt_addr = phys_to_virt(addr);
	*(virt_addr as *mut T) = value;
}

pub fn init() {

}

pub unsafe fn alloc() -> Option<PhysAddr> {
	let freelist_addr = PHYS_FREELIST?;
	let entry = read_phys::<PhysEntry>(freelist_addr);

	PHYS_FREELIST = entry.next;

	Some(freelist_addr)
}

pub unsafe fn free(addr: PhysAddr) {
	assert!(addr.is_aligned(4096));

	let freelist = PHYS_FREELIST;
	let entry = PhysEntry { next: freelist };

	write_phys::<PhysEntry>(addr, entry);
	PHYS_FREELIST = Some(addr);
}