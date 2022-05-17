use crate::platform;
use crate::mmu::{PhysAddr, PagePermission, MapType};
use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::memory::AddressSpace;
use crate::process::{Process, Thread};
use crate::constants::*;
use crate::phys_allocator;

use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use elf_rs::*;

extern "C" {
	static __text_start: i32;
	static __bss_end: i32;
}

// XXX make rust
extern "C" {
	fn user_thread_starter();
	fn invalidate_tlb();
	fn clear_cache_for_address(addr: usize);
}

use crate::arch::context::ExceptionContext;
#[cfg(target_arch = "aarch64")]
fn setup_user_context(process: Arc<Mutex<Box<Process>>>, usermode_pc: usize, usermode_sp: usize) -> Arc<Thread> {
	let new_thread = Arc::new(Thread::new(process.clone()));

	unsafe {
		let mut context_locked = new_thread.context.lock();

		let exc_context_location = new_thread.kernel_stack_top - core::mem::size_of::<ExceptionContext>() - 8; // XXX: align
		let exc_context = &mut *(exc_context_location as *mut ExceptionContext);

		exc_context.regs[31] = usermode_sp;
		exc_context.saved_pc = usermode_pc;
		exc_context.saved_spsr = 0;
		exc_context.saved_tpidr = new_thread.thread_local_location;

		context_locked.regs[30] = user_thread_starter as usize;
		context_locked.regs[31] = exc_context_location;
	}

	process.lock().threads.push(new_thread.clone());
	new_thread
}

#[cfg(target_arch = "x86_64")]
fn setup_user_context(process: Arc<Mutex<Box<Process>>>, usermode_pc: usize, usermode_sp: usize) -> Arc<Thread> {
	unimplemented!();
}

pub fn load_process(elf_buf: &[u8]) -> Arc<Thread> {
	// Load the first process
	let aspace = { 
		let page_table_root = &KERNEL_ADDRESS_SPACE.read().page_table;
		AddressSpace::new(page_table_root.user_process())
	};

	let mut p = Box::new(Process::new(Box::new(aspace)));
	p.use_pages();
	
	let elf = elf_rs::Elf::from_bytes(elf_buf).unwrap();
	if let elf_rs::Elf::Elf64(e) = elf {
		for section in e.section_header_iter() {
			let sh = section.sh;
			if sh.sh_type() == SectionType::SHT_PROGBITS {
				if sh.flags().contains(SectionHeaderFlags::SHF_ALLOC) {
					if sh.flags().contains(SectionHeaderFlags::SHF_EXECINSTR) {
						p.address_space.create(sh.addr() as usize, sh.size() as usize, PagePermission::USER_RWX);
					}
					else {
						p.address_space.create(sh.addr() as usize, sh.size() as usize, PagePermission::USER_READ_WRITE);
					}
					
					// TODO: proper TLB management
					unsafe { invalidate_tlb(); }

					unsafe {
						core::ptr::copy_nonoverlapping(elf_buf.as_ptr().offset(sh.offset() as isize), sh.addr() as *mut u8, sh.size() as usize);
					}

					// TODO: proper cache management
					let section_start: usize = sh.addr() as usize;
					let section_end: usize = section_start + sh.size() as usize;
					for addr in (section_start .. section_end).step_by(64) {
						unsafe { clear_cache_for_address(addr); }
					}
					println!("{:x?}", section);
				}
			}
		}

		let user_code_base = e.header().entry_point() as usize;
		let user_stack_base = 0x40000000;
		let user_stack_size = 0x4000;

		p.address_space.create(user_stack_base, user_stack_size, PagePermission::USER_READ_WRITE);

		let arc = Arc::new(Mutex::new(p));

		let thread = setup_user_context(arc, user_code_base, user_stack_base + user_stack_size);
		return thread
	}
	panic!("Failed to load process??");
}

pub fn setup_physical_allocator(start: usize, end: usize) {
	unsafe {
		let phys_mem_start = platform::PHYS_MEM_BASE;

		let text_start_virt = &__text_start as *const i32 as usize;
		let bss_end_virt = &__bss_end as *const i32 as usize;

		// TODO: This assumes the kernel elf is loaded at the start of physical memory directly, which is kind of nasty.
		let text_start: usize = text_start_virt - KERNEL_BASE + phys_mem_start;
		let bss_end: usize = bss_end_virt - KERNEL_BASE + phys_mem_start;

		for i in (start .. end).step_by(0x1000).rev() {
			if !(i >= text_start && i <= bss_end) {
				phys_allocator::free(PhysAddr(i))
			}
		}
	}
}

pub fn setup_virtual_memory() {
	let page_table_root = &mut KERNEL_ADDRESS_SPACE.write().page_table;

	// map first 4gb into physmap
	page_table_root.map_1gb(PhysAddr(0), PHYSMAP_BASE, PagePermission::KERNEL_RWX, MapType::NormalUncachable);
	page_table_root.map_1gb(PhysAddr(0x40000000), PHYSMAP_BASE + 0x40000000, PagePermission::KERNEL_RWX, MapType::NormalUncachable);
	page_table_root.map_1gb(PhysAddr(0x80000000), PHYSMAP_BASE + 0x80000000, PagePermission::KERNEL_RWX, MapType::NormalUncachable);
	page_table_root.map_1gb(PhysAddr(0xc0000000), PHYSMAP_BASE + 0xc0000000, PagePermission::KERNEL_RWX, MapType::NormalUncachable);

	// map first 4gb into devicemap
	page_table_root.map_1gb(PhysAddr(0), PERIPHERAL_BASE, PagePermission::KERNEL_RWX, MapType::Device);
	page_table_root.map_1gb(PhysAddr(0x40000000), PERIPHERAL_BASE + 0x40000000, PagePermission::KERNEL_RWX, MapType::Device);
	page_table_root.map_1gb(PhysAddr(0x80000000), PERIPHERAL_BASE + 0x80000000, PagePermission::KERNEL_RWX, MapType::Device);
	page_table_root.map_1gb(PhysAddr(0xc0000000), PERIPHERAL_BASE + 0xc0000000, PagePermission::KERNEL_RWX, MapType::Device);

	// map kernel in
	unsafe {
		let text_start_virt = &__text_start as *const i32 as usize;
		let bss_end_virt = &__bss_end as *const i32 as usize;

		let kernel_length = bss_end_virt - text_start_virt;

		// TODO: Figure out the proper physical location of the kernel!
		for i in (0x0000000..kernel_length).step_by(0x200000) {
			page_table_root.map_2mb(PhysAddr(platform::PHYS_MEM_BASE + i), KERNEL_BASE + i, PagePermission::KERNEL_RWX, MapType::NormalCachable);
		}
	}
}