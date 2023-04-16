use crate::arch::context::ThreadContext;
use crate::handle_table::HandleTable;
use crate::memory::AddressSpace;

use alloc::alloc::{alloc, Layout};
use alloc::boxed::Box;
use alloc::sync::Arc;
use atomic_enum::atomic_enum;
use core::sync::atomic::{AtomicUsize, AtomicBool};
use core::sync::atomic::Ordering;
use spin::Mutex;

use intrusive_collections::intrusive_adapter;
use intrusive_collections::{LinkedList, LinkedListAtomicLink};

#[derive(PartialEq)]
#[atomic_enum]
pub enum ThreadState {
    Created,
    Runnable,
    Suspended,
}

pub struct Thread {
    pub all_threads_link: LinkedListAtomicLink,
    pub running_link: LinkedListAtomicLink,
    pub process_link: LinkedListAtomicLink,

    pub id: usize,
    pub state: AtomicThreadState,
    pub context: Mutex<ThreadContext>,

    // static
    pub process: Arc<Mutex<Process>>,
    pub kernel_stack_top: usize,
    pub kernel_stack_size: usize,

    pub is_idle_thread: AtomicBool
}

intrusive_adapter!(pub ThreadProcessAdapter = Arc<Thread>: Thread { process_link: LinkedListAtomicLink });

impl core::fmt::Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_fmt(format_args!("Thread id={}", self.id))
    }
}

#[derive(Debug)]
pub struct Process {
    pub all_processes_link: LinkedListAtomicLink,
    pub id: usize,
    pub address_space: AddressSpace,
    pub threads: LinkedList<ThreadProcessAdapter>,
    pub handle_table: HandleTable,
    pub name: &'static str,
}

intrusive_adapter!(ProcessAdapter = Box<Process>: Process { all_processes_link: LinkedListAtomicLink });

static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);
static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

impl Thread {
    pub fn new(process: Arc<Mutex<Process>>) -> Arc<Thread> {
        let kernel_stack_size = 0x1000;

        let kernel_stack =
            unsafe { alloc(Layout::from_size_align(kernel_stack_size, 0x1000).unwrap()) };

        let thread = Arc::new(Thread {
            all_threads_link: LinkedListAtomicLink::new(),
            running_link: LinkedListAtomicLink::new(),
            process_link: LinkedListAtomicLink::new(),
            id: THREAD_ID.fetch_add(1, Ordering::SeqCst),
            state: AtomicThreadState::new(ThreadState::Created),
            context: Mutex::new(ThreadContext::new()),
            process: process.clone(),
            kernel_stack_top: kernel_stack as *const usize as usize + kernel_stack_size,
            kernel_stack_size: kernel_stack_size,
            is_idle_thread: AtomicBool::new(false)
        });

        process.lock().threads.push_back(thread.clone());
        thread
    }
}

impl Process {
    pub fn new(name: &'static str, aspace: AddressSpace) -> Process {
        let p = Process {
            all_processes_link: LinkedListAtomicLink::new(),
            address_space: aspace,
            threads: LinkedList::new(ThreadProcessAdapter::new()),
            id: PROCESS_ID.fetch_add(1, Ordering::SeqCst),
            handle_table: HandleTable::new(),
            name: name,
        };

        p
    }

    pub fn use_pages(&self) {
        self.address_space.make_active();
    }
}
