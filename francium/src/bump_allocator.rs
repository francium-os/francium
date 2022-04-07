extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};
use crate::constants::*;

use crate::memory::KERNEL_ADDRESS_SPACE;

static mut ALLOC_START: usize = KERNEL_HEAP_BASE;
static mut ALLOC_PRESENT: usize = KERNEL_HEAP_BASE + KERNEL_HEAP_INITIAL_SIZE;

struct BumpAllocator {
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { 
    	// respect layout.size, layout.align
    	let mut start = ALLOC_START;

    	let alignment = layout.align();
    	if start & (alignment-1) != 0 {
    		start += alignment - (start & (alignment-1));
    	}
    	
    	//println!("Bump! {:x} {:x}", ALLOC_START, layout.size());

        ALLOC_START = start;
        ALLOC_START += layout.size();

        if ALLOC_START >= ALLOC_PRESENT {
            let requested_len = ((ALLOC_START - KERNEL_HEAP_BASE) & !(0xfff)) + 0x1000;
            println!("Expanding kernel heap! We want a new size of {:?} bytes.", requested_len);
            {
                let kernel_aspace = &mut KERNEL_ADDRESS_SPACE.write();
                kernel_aspace.expand(KERNEL_HEAP_BASE, requested_len);
            }

            // round up to page
            ALLOC_PRESENT = (ALLOC_START & !0xfff) + 0x1000;
        }

    	return start as *mut u8;
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static BUMP_ALLOCATOR: BumpAllocator = BumpAllocator{};