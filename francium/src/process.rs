use crate::memory::AddressSpace;
use crate::arch::aarch64::context::ThreadContext;
use crate::handle_table::HandleTable;
use crate::mmu::PagePermission;
use alloc::alloc::{alloc, Layout};
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::{Mutex, MutexGuard};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use smallvec::SmallVec;

#[derive(Debug)]
pub enum ThreadState {
	Created,
	Runnable,
	Suspended
}
const TLS_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Thread {
	pub id: usize,
	pub state: ThreadState,
	pub context: Mutex<ThreadContext>,

	// static
	pub process: Arc<Mutex<Box<Process>>>,
	pub kernel_stack_top: usize,
	pub kernel_stack_size: usize,

	pub thread_local: Box<[u8; TLS_SIZE]>,
	pub thread_local_location: usize
}

#[derive(Debug)]
pub struct Process {
	pub id: usize,
	pub address_space: Box<AddressSpace>,
	pub threads: SmallVec<[Arc<Thread>; 1]>,
	pub handle_table: HandleTable,
	pub thread_local_location: usize
}

static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);
static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

// todo: have process keep track of mappings, so we can have not 4k tls..

impl Thread {
	pub fn new(p: Arc<Mutex<Box<Process>>>) -> Thread {
		let kernel_stack_size = 0x1000;

		let kernel_stack = unsafe {
			alloc(Layout::from_size_align(kernel_stack_size, 0x1000).unwrap())
		};

		let thread_local = Box::new([0; TLS_SIZE]);
		let thread_local_location = { 
			let mut locked = p.lock();

			let loc = locked.thread_local_location;
			locked.thread_local_location += TLS_SIZE;

			let phys_page_loc = locked.address_space.page_table.virt_to_phys(thread_local.as_ptr() as usize).unwrap();
			locked.address_space.alias(phys_page_loc, loc, TLS_SIZE, PagePermission::USER_READ_WRITE);

			loc
		};

		Thread {
			id: THREAD_ID.fetch_add(1, Ordering::SeqCst),
			state: ThreadState::Created,
			context: Mutex::new(ThreadContext::new()),
			process: p,
			kernel_stack_top: kernel_stack as *const usize as usize + kernel_stack_size,
			kernel_stack_size: kernel_stack_size,
			thread_local: thread_local,
			thread_local_location: thread_local_location
		}
	}
}

impl Process {
	pub fn new(aspace: Box<AddressSpace>) -> Process {
		let p = Process {
			address_space: aspace,
			threads: SmallVec::new(),
			id: PROCESS_ID.fetch_add(1, Ordering::SeqCst),
			handle_table: HandleTable::new(),
			thread_local_location: 0x50000000
		};

		p
	}

	pub fn use_pages(&self) {
		self.address_space.make_active();
	}
}

// see also: force_unlock_mutex in scheduler
extern "C" {
	fn restore_thread_context(ctx: &ThreadContext, mutex: usize);
}

pub fn force_switch_to(thread: Arc<Thread>) {
	thread.process.lock().use_pages();

	let thread_context = MutexGuard::leak(thread.context.lock());
	unsafe {
		restore_thread_context(thread_context, &thread.context as *const Mutex<ThreadContext> as usize);
	}
}