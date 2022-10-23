use crate::platform;
use crate::mmu::{PhysAddr, PagePermission, MapType};
use crate::memory::KERNEL_ADDRESS_SPACE;
use crate::memory::AddressSpace;
use crate::process::{Process, Thread};
use crate::constants::*;
use crate::phys_allocator;
use crate::arch::mmu::{get_current_page_table, invalidate_tlb_for_range};
use crate::arch::cache::clear_cache_for_address;

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

		context_locked.regs[30] = user_thread_starter as usize;
		context_locked.regs[31] = exc_context_location;
	}

	process.lock().threads.push(new_thread.clone());
	new_thread
}

#[cfg(target_arch = "x86_64")]
fn setup_user_context(process: Arc<Mutex<Box<Process>>>, usermode_pc: usize, usermode_sp: usize) -> Arc<Thread> {
	let new_thread = Arc::new(Thread::new(process.clone()));

	unsafe {
		let mut context_locked = new_thread.context.lock();

		let exc_context_location = new_thread.kernel_stack_top - core::mem::size_of::<ExceptionContext>(); // XXX: align
		let exc_context = &mut *(exc_context_location as *mut ExceptionContext);

		exc_context.regs.rsp = usermode_sp;
		exc_context.regs.rip = usermode_pc;
		exc_context.regs.cs = 0x18 | 3;
		exc_context.regs.ss = 0x20 | 3;

		exc_context.regs.rax = 1;
		exc_context.regs.rbx = 2;
		exc_context.regs.rcx = 3;
		exc_context.regs.rdx = 4;
		exc_context.regs.rbp = 5;
		exc_context.regs.rsi = 6;
		exc_context.regs.rdi = 7;

		exc_context.regs.r8 = 8;
		exc_context.regs.r9 = 9;
		exc_context.regs.r10 = 10;
		exc_context.regs.r11 = 11;
		exc_context.regs.r12 = 12;
		exc_context.regs.r13 = 13;
		exc_context.regs.r14 = 14;
		exc_context.regs.r15 = 15;

		context_locked.regs.rip = user_thread_starter as usize;
		context_locked.regs.rsp = exc_context_location;
	}

	process.lock().threads.push(new_thread.clone());
	new_thread
}

pub fn load_process(elf_buf: &[u8], name: &'static str) -> Arc<Thread> {
	println!("loading {}", name);

	// Load the first process
	let aspace = { 
		let page_table_root = &KERNEL_ADDRESS_SPACE.read().page_table;
		AddressSpace::new(page_table_root.user_process())
	};

	let mut p = Box::new(Process::new(name, Box::new(aspace)));
	p.use_pages();
	
	let elf = Elf::from_bytes(elf_buf).unwrap();
	if let Elf::Elf64(e) = elf {
		for phdr in e.program_header_iter() {
			let ph = phdr.ph;
			if ph.ph_type() == ProgramType::LOAD {
				let section_start: usize = ph.vaddr() as usize;
				let section_size: usize = ph.memsz() as usize;
				let section_size_aligned: usize = (section_size + (PAGE_SIZE-1)) & !(PAGE_SIZE-1);

				if (ph.flags() & 1) == 1 { // TODO: where did `1` come from?
					p.address_space.create(section_start, section_size_aligned, PagePermission::USER_RWX);
				} else {
					p.address_space.create(section_start, section_size_aligned, PagePermission::USER_READ_WRITE);
				}

				// TODO: proper TLB management
				unsafe { invalidate_tlb_for_range(section_start, section_size_aligned); }

				if ph.filesz() != 0 {
					unsafe {
						core::ptr::copy_nonoverlapping(elf_buf.as_ptr().offset(ph.offset() as isize), ph.vaddr() as *mut u8, ph.filesz() as usize);
					}
				}

				if ph.filesz() < ph.memsz() {
					// BSS section
					unsafe {
						core::ptr::write_bytes((ph.vaddr() + ph.filesz()) as *mut u8, 0, (ph.memsz()-ph.filesz()) as usize);
					}
				}
				
				// TODO: proper cache management
				let section_end: usize = section_start + ph.memsz() as usize;
				for addr in (section_start .. section_end).step_by(64) {
					unsafe { clear_cache_for_address(addr); }
				}
			} else if ph.ph_type() == ProgramType::Unknown(7) { // unk(7) = TLS
				// load TLS template
				let tls_start: usize = ph.offset() as usize;
				let tls_end: usize = ph.offset() as usize + ph.filesz() as usize;
				p.thread_local_template.extend(&elf_buf[tls_start..tls_end]);
				
				let align: usize = ph.align() as usize;
				let memsz_aligned = (ph.memsz() as usize + align-1) & !(align-1);
				p.thread_local_template.resize(memsz_aligned, 0);

				if ph.memsz() as usize > crate::process::TLS_SIZE {
					panic!("no");
				}
			}
		}

		let user_code_base = e.header().entry_point() as usize;
		let user_stack_base = 0x40000000;
		let user_stack_size = 0x4000;

		p.address_space.create(user_stack_base, user_stack_size, PagePermission::USER_READ_WRITE);

		let arc = Arc::new(Mutex::new(p));

		// Fill out argv/etc.
		// Currently: one argv, a stub.

		let argc: isize = 1;
		let argv: [&'static str; 1] = ["test"];
		let envp: [&'static str; 0] = [];
		let auxv_entries: [(usize, usize); 0] = [];

		// XXX: ptr size?
		let argv_size: usize = 8 + (argc as usize + 1) * 8;
		let env_size = (envp.len() + 1) * 8;
		let auxv_size = (auxv_entries.len() + 1) * 16;

		let strings_len = argv.iter().map(|x| x.len() + 1).sum::<usize>() + envp.iter().map(|x| x.len() + 1).sum::<usize>();

		let new_stack = user_stack_base + user_stack_size;
		let auxv_base = new_stack - (argv_size + env_size + auxv_size + strings_len + 8);

		// XXX This is bad.
		// XXX Todo: align?

		unsafe {
			let mut auxv = auxv_base;
			let mut strings = auxv_base + argv_size + env_size + auxv_size;

			core::ptr::copy_nonoverlapping(&argc as *const isize, auxv as *mut isize, 1);
			auxv += core::mem::size_of::<isize>();

			for i in 0..argv.len() {
				core::ptr::copy_nonoverlapping(&strings as *const usize, auxv as *mut usize, 1);
				auxv += core::mem::size_of::<usize>();

				let s = argv[i];
				core::ptr::copy_nonoverlapping(s.as_ptr(), strings as *mut u8, s.len());
				strings += s.len();
				*(strings as *mut u8) = 0;
				strings += 1;
			}

			core::ptr::write_bytes(auxv as *mut usize, 0, 1);
			auxv += core::mem::size_of::<usize>();

			for i in 0..envp.len() {
				let s = envp[i];
				core::ptr::copy_nonoverlapping(s.as_ptr(), strings as *mut u8, s.len());
				strings += s.len();
				*(strings as *mut u8) = 0;
				strings += 1;
			}

			core::ptr::write_bytes(auxv as *mut usize, 0, 1);
			auxv += core::mem::size_of::<usize>();

			for _ in 0..auxv_entries.len() {
				unimplemented!();
			}
			
			core::ptr::write_bytes(auxv as *mut usize, 0, 2);
			//auxv += core::mem::size_of::<usize>() * 2;

			core::ptr::write_bytes(strings as *mut usize, 0, 1);
			//strings += core::mem::size_of::<usize>() * 1;
		}

		let thread = setup_user_context(arc, user_code_base, auxv_base);
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

	// hack
	unsafe {
        // Some bootloaders (cough x86) might not put us in the right place.
        // Figure it out.

		let current_pages = get_current_page_table();
		//let kernel_phys_base = current_pages.virt_to_phys(KERNEL_BASE).unwrap().0;

		//let text_start_virt = &__text_start as *const i32 as usize;
		let bss_end_virt = &__bss_end as *const i32 as usize;

		// use KERNEL_BASE instead of text_start because pi4 has text_start at 0x80000
		let kernel_length = bss_end_virt - KERNEL_BASE;

		for i in (0x0000000..kernel_length).step_by(0x1000) {
			page_table_root.map_4k(current_pages.virt_to_phys(KERNEL_BASE+i).unwrap(), KERNEL_BASE + i, PagePermission::KERNEL_RWX, MapType::NormalCachable);
		}
	}
}
