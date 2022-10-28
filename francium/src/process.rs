use crate::memory::AddressSpace;
use crate::arch::context::ThreadContext;
use crate::handle_table::HandleTable;
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

#[derive(Debug)]
pub struct Thread {
	pub id: usize,
	pub state: ThreadState,
	pub context: Mutex<ThreadContext>,

	// static
	pub process: Arc<Mutex<Box<Process>>>,
	pub kernel_stack_top: usize,
	pub kernel_stack_size: usize,
}

#[derive(Debug)]
pub struct Process {
	pub id: usize,
	pub address_space: Box<AddressSpace>,
	pub threads: SmallVec<[Arc<Thread>; 1]>,
	pub handle_table: HandleTable,
	pub name: &'static str
}

static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);
static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

impl Thread {
	pub fn new(p: Arc<Mutex<Box<Process>>>) -> Thread {
		let kernel_stack_size = 0x1000;

		let kernel_stack = unsafe {
			alloc(Layout::from_size_align(kernel_stack_size, 0x1000).unwrap())
		};

		
		Thread {
			id: THREAD_ID.fetch_add(1, Ordering::SeqCst),
			state: ThreadState::Created,
			context: Mutex::new(ThreadContext::new()),
			process: p,
			kernel_stack_top: kernel_stack as *const usize as usize + kernel_stack_size,
			kernel_stack_size: kernel_stack_size,
		}
	}
}

impl Process {
	pub fn new(name: &'static str, aspace: Box<AddressSpace>) -> Process {
		let p = Process {
			address_space: aspace,
			threads: SmallVec::new(),
			id: PROCESS_ID.fetch_add(1, Ordering::SeqCst),
			handle_table: HandleTable::new(),
			name: name
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
		#[cfg(target_arch = "x86_64")]
		scheduler::set_current_thread_state(thread.kernel_stack_top, 0);
		setup_initial_thread_context(thread_context, &thread.context as *const Mutex<ThreadContext> as usize);
	}
}
