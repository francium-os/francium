use crate::memory::AddressSpace;
use crate::arch::context::ThreadContext;
use crate::handle_table::HandleTable;
use crate::mmu::PagePermission;
use crate::scheduler;

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
const TLS_SIZE: usize = 512;

#[derive(Debug)]
pub struct Thread {
	pub id: usize,
	pub state: ThreadState,
	pub context: Mutex<ThreadContext>,

	// static
	pub process: Arc<Mutex<Box<Process>>>,
	pub kernel_stack_top: usize,
	pub kernel_stack_size: usize,

	// safety: dude trust me
	pub thread_local: &'static mut [u8; TLS_SIZE],
	pub thread_local_location: usize
}

#[derive(Debug)]
pub struct Process {
	pub id: usize,
	pub address_space: Box<AddressSpace>,
	pub threads: SmallVec<[Arc<Thread>; 1]>,
	pub handle_table: HandleTable,

	pub thread_local_start: usize,
	pub thread_local_location: usize,
	pub thread_local_size: usize
}

static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);
static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

impl Thread {
	pub fn new(p: Arc<Mutex<Box<Process>>>) -> Thread {
		let kernel_stack_size = 0x1000;

		let kernel_stack = unsafe {
			alloc(Layout::from_size_align(kernel_stack_size, 0x1000).unwrap())
		};

		let thread_local_location = { 
			let mut locked = p.lock();

			let loc = locked.thread_local_location;

			if locked.thread_local_location + TLS_SIZE >= locked.thread_local_start + locked.thread_local_size {
				locked.thread_local_size += 0x1000;
				let tls_start = locked.thread_local_start;
				let new_size = locked.thread_local_size;
				locked.address_space.expand(tls_start, new_size);
			}
			locked.thread_local_location += TLS_SIZE;

			loc
		};

		let thread_local = unsafe {
			(thread_local_location as *mut [u8; TLS_SIZE]).as_mut().unwrap()
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
	pub fn new(mut aspace: Box<AddressSpace>) -> Process {
		let thread_local_start = 0x50000000;
		aspace.create(thread_local_start, 0x1000, PagePermission::USER_READ_WRITE);

		let p = Process {
			address_space: aspace,
			threads: SmallVec::new(),
			id: PROCESS_ID.fetch_add(1, Ordering::SeqCst),
			handle_table: HandleTable::new(),
			thread_local_start: thread_local_start,
			thread_local_location: thread_local_start,
			thread_local_size: 0x1000
		};

		p
	}

	pub fn use_pages(&self) {
		self.address_space.make_active();
	}
}

// see also: force_unlock_mutex in scheduler
extern "C" {
	fn setup_initial_thread_context(ctx: &ThreadContext, mutex: usize);
}

pub fn force_switch_to(thread: Arc<Thread>) {
	thread.process.lock().use_pages();

	let thread_context = MutexGuard::leak(thread.context.lock());
	unsafe {
		scheduler::set_current_thread_stack(thread.kernel_stack_top);
		setup_initial_thread_context(thread_context, &thread.context as *const Mutex<ThreadContext> as usize);
	}
}
