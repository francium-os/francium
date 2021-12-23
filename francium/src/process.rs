use crate::memory::AddressSpace;
use crate::arch::aarch64::context::ProcessContext;
use crate::aarch64::context::ExceptionContext;
use alloc::alloc::{alloc, Layout};
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use crate::handle_table::HandleTable;

#[derive(Debug)]
pub enum ProcessState {
	Created,
	Runnable,
	Suspended
}

#[derive(Debug)]
pub struct Process {
	pub address_space: Box<AddressSpace>,
	pub context: ProcessContext,
	pub state: ProcessState,
	pub id: usize,
	pub handle_table: HandleTable,
	pub kernel_stack_top: usize,
	pub kernel_stack_size: usize
}

static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);

extern "C" {
	fn user_thread_starter();
}

impl Process {
	pub fn new(aspace: Box<AddressSpace>) -> Process {
		let kernel_stack_size = 0x1000;

		let kernel_stack = unsafe {
			alloc(Layout::from_size_align(kernel_stack_size, 0x1000).unwrap())
		};

		let p = Process {
			address_space: aspace,
			context: ProcessContext::new(),
			state: ProcessState::Created,
			id: PROCESS_ID.fetch_add(1, Ordering::SeqCst),
			handle_table: HandleTable::new(),
			kernel_stack_top: kernel_stack as *const usize as usize + kernel_stack_size,
			kernel_stack_size: kernel_stack_size,
		};

		p
	}

	pub fn setup_user_context(&mut self, usermode_pc: usize, usermode_sp: usize) {
		unsafe {
			let exc_context_location = self.kernel_stack_top - core::mem::size_of::<ExceptionContext>();

			let exc_context = &mut *(exc_context_location as *mut ExceptionContext);

			exc_context.regs[31] = usermode_sp;
			exc_context.saved_pc = usermode_pc;
			exc_context.saved_spsr = 0;

			self.context.regs[30] = user_thread_starter as usize;
			self.context.regs[31] = exc_context_location;
		}
	}

	pub fn use_pages(&self) {
		self.address_space.make_active();
	}
}

pub fn force_switch_to(locked: Arc<Mutex<Box<Process>>>) {
	let process_context = { 
		let p = locked.lock();
		p.address_space.make_active();
		p.context.clone()
	};
	unsafe {
		process_context.switch();
	}
}