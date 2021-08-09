extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};
use crate::constants::*;

static mut ALLOC_START: usize = KERNEL_HEAP_BASE;
static mut ALLOC_PRESENT: usize = KERNEL_HEAP_BASE;

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
    	
    	ALLOC_START = start;
        ALLOC_START += layout.size();

        if ALLOC_START >= ALLOC_PRESENT {
            let len = ALLOC_START - ALLOC_PRESENT;
            panic!("We need to make up some memory {:x} {:x} {:x}", ALLOC_START, ALLOC_PRESENT, len);

            ALLOC_PRESENT += 0x1000;
        }

    	return start as *mut u8;
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static BUMP_ALLOCATOR: BumpAllocator = BumpAllocator{};