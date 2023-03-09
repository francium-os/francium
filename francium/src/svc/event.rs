use crate::drivers::InterruptController;
use crate::handle::HandleObject;
use crate::platform::DEFAULT_INTERRUPT;
use crate::scheduler;
use crate::waitable::Waiter;
use alloc::sync::Arc;
use common::os_error::{Module, Reason, ResultCode, RESULT_OK};
use spin::Mutex;

#[derive(Debug)]
pub struct Event {
    w: Waiter,
}

impl Event {
    fn new() -> Event {
        Event { w: Waiter::new() }
    }
}

// that's it
// ez

pub fn svc_create_event() -> (ResultCode, u32) {
    let ev = Event::new();

    let proc_locked = scheduler::get_current_process();
    let mut process = proc_locked.lock();

    let handle_value = process
        .handle_table
        .get_handle(HandleObject::Event(Arc::new(ev)));

    (RESULT_OK, handle_value)
}

const NO_EVENT: Option<Arc<Event>> = None;
static EVENT_TABLE: Mutex<[Option<Arc<Event>>; 128]> = Mutex::new([NO_EVENT; 128]);

pub fn dispatch_interrupt_event(index: usize) {
    if let Some(ev) = &EVENT_TABLE.lock()[index] {
        ev.w.signal_one();
    }
}

pub fn svc_bind_interrupt(h: u32, index: usize) -> ResultCode {
    let proc_locked = scheduler::get_current_process();
    let process = proc_locked.lock();

    if let HandleObject::Event(ev) = process.handle_table.get_object(h) {
        let mut lock = EVENT_TABLE.lock();
        if let None = lock[index] {
            lock[index] = Some(ev);
            DEFAULT_INTERRUPT.lock().enable_interrupt(index as u32);

            RESULT_OK
        } else {
            ResultCode::new(Module::Kernel, Reason::Unknown)
        }
    } else {
        ResultCode::new(Module::Kernel, Reason::InvalidHandle)
    }
}

pub fn svc_unbind_interrupt(h: u32, index: usize) -> ResultCode {
    let proc_locked = scheduler::get_current_process();
    let process = proc_locked.lock();

    if let HandleObject::Event(ev) = process.handle_table.get_object(h) {
        let mut lock = EVENT_TABLE.lock();
        if let Some(x) = &lock[index] {
            lock[index] = None;
            DEFAULT_INTERRUPT.lock().disable_interrupt(index as u32);
        } else {
            return ResultCode::new(Module::Kernel, Reason::Unknown);
        }
    }

    ResultCode::new(Module::Kernel, Reason::InvalidHandle)
}
