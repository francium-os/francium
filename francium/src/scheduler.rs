use alloc::sync::Arc;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use spin::{Mutex, MutexGuard};

use crate::arch::context::ThreadContext;
use crate::process::{Process, Thread, ThreadState};

use intrusive_collections::intrusive_adapter;
use intrusive_collections::{LinkedList, LinkedListAtomicLink};

use log::trace;

intrusive_adapter!(pub ThreadAdapter = Arc<Thread>: Thread { all_threads_link: LinkedListAtomicLink });
intrusive_adapter!(pub ThreadRunnableAdapter = Arc<Thread>: Thread { running_link: LinkedListAtomicLink });

// TODO: idle_thread is an option because constructing an Arc needs the heap up.

pub struct Scheduler {
    pub threads: LinkedList<ThreadAdapter>,
    pub runnable_threads: LinkedList<ThreadRunnableAdapter>,
    
    // XXX: PER CPU (ie remove from scheduler!)
    pub current_thread: Option<Arc<Thread>>,
    pub idle_thread: Option<Arc<Thread>>,
}

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}
extern "C" {
    fn switch_thread_asm(
        from_context: *mut ThreadContext,
        to_context: *const ThreadContext,
        from: usize,
        to: usize,
    ) -> usize;
}

#[no_mangle]
pub extern "C" fn force_unlock_mutex(mutex: NonNull<Mutex<ThreadContext>>) {
    unsafe {
        mutex.as_ref().force_unlock();
    }
}

#[cfg(target_arch = "aarch64")]
fn set_thread_context_tag(p: &Arc<Thread>, tag: usize) {
    p.context.lock().regs[0] = tag;
}

#[cfg(target_arch = "x86_64")]
fn set_thread_context_tag(p: &Arc<Thread>, tag: usize) {
    p.context.lock().regs.rax = tag;
}

#[cfg(target_arch = "x86_64")]
use crate::arch;

#[cfg(target_arch = "x86_64")]
pub unsafe fn set_current_thread_state(kernel_stack: usize, tls: usize) {
    crate::per_cpu::get().saved_kernel_stack = kernel_stack;
    arch::x86_64::gdt::TSS_STORAGE.rsp0 = kernel_stack as u64;
    arch::msr::write_fs_base(tls);
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            threads: LinkedList::new(ThreadAdapter::new()),
            runnable_threads: LinkedList::new(ThreadRunnableAdapter::new()),
            current_thread: None,
            idle_thread: None,
        }
    }

    fn set_idle_thread(&mut self, thread: Arc<Thread>) {
        self.idle_thread = Some(thread);
    }

    fn get_current_thread(&self) -> Arc<Thread> {
        self.current_thread.as_ref().unwrap().clone()
    }

    fn switch_thread(&mut self, from: &Arc<Thread>, to: &Arc<Thread>) -> usize {
        trace!("Switch from {} to {}", from.id, to.id);

        if from.id == to.id {
            // don't do this, it'll deadlock
            //panic!("Trying to switch to the same thread!");
            return 0;
        }

        let idle_thread = self.idle_thread.as_ref().unwrap();
        // TODO: see comment in wake, this kind of sucks
        if from.id == idle_thread.id {
            // We are switching off the idle thread. Suspend it.

            idle_thread
                .state
                .store(ThreadState::Suspended, Ordering::Release);
        }

        self.current_thread = Some(to.clone());

        // TODO: wow, this sucks
        {
            unsafe {
                // TODO: lol
                SCHEDULER.force_unlock();
            }

            {
                to.process.lock().use_pages();
            }

            let from_context_locked = MutexGuard::leak(from.context.lock());
            let to_context_locked = MutexGuard::leak(to.context.lock());

            let from_context_ptr = &from.context as *const Mutex<ThreadContext>;
            let to_context_ptr = &to.context as *const Mutex<ThreadContext>;

            unsafe {
                #[cfg(target_arch = "x86_64")]
                set_current_thread_state(to.kernel_stack_top, to_context_locked.regs.fs);

                return switch_thread_asm(
                    from_context_locked,
                    to_context_locked,
                    from_context_ptr as usize,
                    to_context_ptr as usize,
                );
            }
        }
    }

    pub fn advance_to_next_thread(&mut self) -> Arc<Thread> {
        let mut cursor = unsafe {
            self.runnable_threads.cursor_from_ptr(Arc::<Thread>::as_ptr(
                &self.current_thread.as_ref().unwrap(),
            ))
        };
        cursor.move_next();
        if cursor.is_null() {
            cursor.move_next();
        }

        let new_thread = cursor.clone_pointer().unwrap();
        self.current_thread = Some(new_thread.clone());
        new_thread
    }

    pub fn tick(&mut self) {
        // XXX TODO: O(h no)
        if self.runnable_threads.iter().count() == 0 {
            return;
        }

        // do the thing
        let this_thread = self.get_current_thread();
        let next: Option<Arc<Thread>> = if this_thread.is_idle_thread.load(Ordering::Acquire) {
            trace!("Runnable threads: {:?}", self.runnable_threads);
            let cursor = self.runnable_threads.front();
            if cursor.is_null() {
                panic!("No more runnable threads!");
            } else {
                Some(cursor.clone_pointer().unwrap())
            }
        }
        else {
            Some(self.advance_to_next_thread())
        };

        if let Some(next_thread) = next {
            self.switch_thread(&this_thread, &next_thread);
        }
    }

    pub fn suspend(&mut self, thread: &Arc<Thread>) -> usize {
        if thread.state.load(Ordering::Acquire) == ThreadState::Runnable {
            thread
                .state
                .store(ThreadState::Suspended, Ordering::Release);

            
            let current_thread = self.get_current_thread();
            let current_id = current_thread.id;

            trace!(
                "Suspending thread {} ({})",
                current_id,
                current_thread.process.lock().name
            );

            if current_thread.is_idle_thread.load(Ordering::Acquire) {
                panic!("Tried to suspend the idle thread");
            }

            // Safety: thread is runnable and not the idle thread
            let mut cursor = unsafe {
                self.runnable_threads
                    .cursor_mut_from_ptr(Arc::<Thread>::as_ptr(&current_thread))
            };

            // Cursor now points to old thread
            cursor.remove();
            let next_thread = if cursor.is_null() {
                cursor.move_next();

                // If list is empty, it will still be on the null element.
                if cursor.is_null() {
                    self.idle_thread.as_ref().unwrap().clone()
                } else {
                    cursor.as_cursor().clone_pointer().unwrap()
                }
            } else {
                cursor.as_cursor().clone_pointer().unwrap()
            };

            // If we got switched out, switch to the new current thread.
            if current_id == thread.id {
                return self.switch_thread(thread, &next_thread);
            }
        } else {
            panic!(
                "Invalid thread state {:?}",
                thread.state.load(Ordering::Acquire)
            );
        }

        0
    }

    pub fn wake(&mut self, thread: &Arc<Thread>, tag: usize) {
        if thread.state.load(Ordering::Acquire) != ThreadState::Runnable {
            trace!(
                "Waking thread {:?} ({})",
                thread.id,
                thread.process.lock().name
            );

            thread.state.store(ThreadState::Runnable, Ordering::Release);
            // set x0 of the thread context
            set_thread_context_tag(thread, tag);
            self.runnable_threads.push_back(thread.clone());

            // TODO: I tried to add an optimization to immediately suspend the idle thread if its running.
            // but calling switch_thread in wake breaks things pretty badly
            // self.switch_thread(&self.get_current_thread(), thread);
        } else {
            // This should be OK, hopefully.
            trace!("Trying to re-wake thread {:?}!", thread.id);
        }
    }

    pub fn terminate_current_thread(&mut self) {
        let current_thread = self.get_current_thread();

        if current_thread.is_idle_thread.load(Ordering::Acquire) {
            panic!("Tried to terminate the idle thread");
        }

        // Safety: Current thread is a thread
        let mut cursor = unsafe {
            self.threads
                .cursor_mut_from_ptr(Arc::<Thread>::as_ptr(&current_thread))
        };
        let this_thread = cursor.remove().unwrap();

        self.suspend(&this_thread);
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn idle_thread_func() {
    loop {
        core::arch::asm!("wfe");
    }
}

// why 2? https://github.com/rust-lang/rust/issues/94426
#[naked]
#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn idle_thread_func() {
    core::arch::asm!("2: hlt; jmp 2b", options(noreturn));
}

// Set up the idle thread.
pub fn init() {
    use crate::memory::AddressSpace;
    use crate::KERNEL_ADDRESS_SPACE;

    let mut sched = SCHEDULER.lock();
    let aspace = {
        let page_table_root = &KERNEL_ADDRESS_SPACE.read().page_table;
        AddressSpace::new(page_table_root.user_process())
    };

    let idle_process = Arc::new(Mutex::new(Process::new("idle", aspace)));
    let idle_thread = Thread::new(idle_process);
    idle_thread.is_idle_thread.store(true, Ordering::Release);

    idle_thread
        .state
        .store(ThreadState::Runnable, Ordering::Release);

    crate::init::setup_thread_context(
        &idle_thread,
        idle_thread_func as usize,
        idle_thread.kernel_stack_top,
        true,
    );

    sched.threads.push_back(idle_thread.clone());
    sched.set_idle_thread(idle_thread);
}

pub fn tick() {
    let mut sched = SCHEDULER.lock();
    sched.tick();
}

pub fn register_thread(thread: Arc<Thread>) {
    let mut sched = SCHEDULER.lock();
    thread.state.store(ThreadState::Runnable, Ordering::Release);

    sched.threads.push_back(thread.clone());
    sched.runnable_threads.push_back(thread);
}

pub fn get_current_thread() -> Arc<Thread> {
    let sched = SCHEDULER.lock();
    sched.get_current_thread()
}

pub fn get_current_process() -> Arc<Mutex<Process>> {
    get_current_thread().process.clone()
}

pub fn suspend_process(p: Arc<Thread>) {
    let mut sched = SCHEDULER.lock();
    sched.suspend(&p);
}

pub fn suspend_current_thread() -> usize {
    let mut sched = SCHEDULER.lock();
    let curr = sched.get_current_thread();

    return sched.suspend(&curr);
}

pub fn wake_thread(p: &Arc<Thread>, tag: usize) {
    let mut sched = SCHEDULER.lock();
    sched.wake(p, tag);
}

pub fn terminate_current_thread() {
    let mut sched = SCHEDULER.lock();
    sched.terminate_current_thread();
}

pub fn terminate_current_process() {
    let mut sched = SCHEDULER.lock();
    let current_thread = sched.get_current_thread();
    let current_process = current_thread.process.clone();
    let process = current_process.lock();

    let mut cursor = process.threads.front();
    while !cursor.is_null() {
        let thread = cursor.get().unwrap();
        if thread.id != current_thread.id {
            sched.suspend(&cursor.clone_pointer().unwrap());
        }
        cursor.move_next();
    }

    sched.terminate_current_thread();
}

// see also: force_unlock_mutex
extern "C" {
    fn setup_initial_thread_context(ctx: &ThreadContext, mutex: usize);
}

pub fn force_switch_to(thread: Arc<Thread>) {
    {
        thread.state.store(ThreadState::Runnable, Ordering::Release);
        let mut sched = SCHEDULER.lock();
        // TODO: assert in runnable?
        sched.current_thread = Some(thread.clone());
    }

    thread.process.lock().use_pages();

    let thread_context = MutexGuard::leak(thread.context.lock());
    unsafe {
        #[cfg(target_arch = "x86_64")]
        set_current_thread_state(thread.kernel_stack_top, 0);
        setup_initial_thread_context(
            thread_context,
            &thread.context as *const Mutex<ThreadContext> as usize,
        );
    }
}
