use simple_chunk_allocator::{heap, heap_bitmap, GlobalChunkAllocator, PageAligned};
static mut HEAP: PageAligned<[u8; 1048576]> = heap!();
static mut HEAP_BITMAP: PageAligned<[u8; 512]> = heap_bitmap!();

#[global_allocator]
static ALLOCATOR: GlobalChunkAllocator =
    unsafe { GlobalChunkAllocator::new(HEAP.deref_mut_const(), HEAP_BITMAP.deref_mut_const()) };