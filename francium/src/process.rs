use crate::memory::AddressSpace;
use crate::arch::context::ThreadContext;
use crate::handle_table::HandleTable;

use alloc::alloc::{alloc, Layout};
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use smallvec::SmallVec;
use atomic_enum::atomic_enum;

#[atomic_enum]
pub enum ThreadState {
	Created,
	Runnable,
	Suspended
}

pub struct Thread {
	pub id: usize,
	pub state: AtomicThreadState,
	pub context: Mutex<ThreadContext>,

	// static
	pub process: Arc<Mutex<Box<Process>>>,
	pub kernel_stack_top: usize,
	pub kernel_stack_size: usize,
}

impl core::fmt::Debug for Thread
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		f.write_fmt(format_args!("Thread id={}", self.id))
	}
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
	pub fn new(p: Arc<Mutex<Box<Process>>>) -> Arc<Thread> {
		let kernel_stack_size = 0x1000;

		let kernel_stack = unsafe {
			alloc(Layout::from_size_align(kernel_stack_size, 0x1000).unwrap())
		};

		let thread = Arc::new(Thread {
			id: THREAD_ID.fetch_add(1, Ordering::SeqCst),
			state: AtomicThreadState::new(ThreadState::Created),
			context: Mutex::new(ThreadContext::new()),
			process: p.clone(),
			kernel_stack_top: kernel_stack as *const usize as usize + kernel_stack_size,
			kernel_stack_size: kernel_stack_size,
		});

		p.lock().threads.push(thread.clone());
		thread
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