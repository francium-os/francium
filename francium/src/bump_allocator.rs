extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};
use crate::constants::*;

static mut ALLOC_START: usize = KERNEL_HEAP_BASE;
struct BumpAllocator {
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { 
        unimplemented!();

    	// respect layout.size, layout.align
    	let mut start = ALLOC_START;

    	let alignment = layout.align();
    	if start & (alignment-1) != 0 {
    		start += alignment - (start & (alignment-1));
    	}
    	
    	start += layout.size();
    	ALLOC_START = start;

    	return start as *mut u8;
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static BUMP_ALLOCATOR: BumpAllocator = BumpAllocator{};