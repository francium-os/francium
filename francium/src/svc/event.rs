use crate::drivers::InterruptDistributor;
use crate::handle::HandleObject;
use crate::platform::{INTERRUPT_CONTROLLER, INTERRUPT_DISTRIBUTOR};
use crate::scheduler;
use crate::waitable::{Waitable, Waiter};
use alloc::sync::Arc;
use common::os_error::{Module, Reason, ResultCode, RESULT_OK};
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;
use francium_drivers::InterruptController;
use spin::Mutex;

#[derive(Debug)]
pub struct Event {
    pub interrupt: AtomicU32,
    w: Waiter,
}

impl Event {
    fn new() -> Event {
        Event {
            interrupt: AtomicU32::new(0),
            w: Waiter::new(),
        }
    }
}

impl Waitable for Event {
    fn get_waiter(&self) -> &Waiter {
        &self.w
    }
}

pub fn svc_create_event() -> (ResultCode, u32) {
    let ev = Event::new();

    let proc_locked = scheduler::get_current_process();
    let mut process = proc_locked.lock();

    let handle_value = process
        .handle_table
        .get_handle(HandleObject::Event(Arc::new(ev)));

    (RESULT_OK, handle_value)
}

pub fn svc_signal_event(h: u32) -> ResultCode {
    let proc_locked = scheduler::get_current_process();
    let process = proc_locked.lock();

    if let HandleObject::Event(ev) = process.handle_table.get_object(h) {
        ev.w.signal_one(true);
        RESULT_OK
    } else {
        ResultCode::new(Module::Kernel, Reason::InvalidHandle)
    }
}

pub fn svc_clear_event(h: u32) -> ResultCode {
    let proc_locked = scheduler::get_current_process();
    let process = proc_locked.lock();

    if let HandleObject::Event(ev) = process.handle_table.get_object(h) {
        ev.w.clear();

        let interrupt_id = ev.interrupt.load(Ordering::Acquire);
        if interrupt_id != 0 {
            log::trace!("clearing interrupt event {}", interrupt_id);
            INTERRUPT_CONTROLLER.lock().ack_interrupt(interrupt_id);
        }
        RESULT_OK
    } else {
        ResultCode::new(Module::Kernel, Reason::InvalidHandle)
    }
}

const NO_EVENT: Option<Arc<Event>> = None;
pub static INTERRUPT_EVENT_TABLE: Mutex<[Option<Arc<Event>>; 128]> = Mutex::new([NO_EVENT; 128]);

pub fn dispatch_interrupt_event(index: usize) -> bool {
    if let Some(ev) = &INTERRUPT_EVENT_TABLE.lock()[index] {
        unsafe {
            INTERRUPT_EVENT_TABLE.force_unlock();
        }
        ev.w.signal_one(true);
        true
    } else {
        false
    }
}

pub fn svc_bind_interrupt(h: u32, index: usize) -> ResultCode {
    let proc_locked = scheduler::get_current_process();
    let process = proc_locked.lock();

    if let HandleObject::Event(ev) = process.handle_table.get_object(h) {
        let mut lock = INTERRUPT_EVENT_TABLE.lock();
        if let None = lock[index] {
            ev.interrupt.store(index as u32, Ordering::Release);
            lock[index] = Some(ev);
            INTERRUPT_DISTRIBUTOR.lock().enable_interrupt(index as u32);

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

    if let HandleObject::Event(_ev) = process.handle_table.get_object(h) {
        let mut lock = INTERRUPT_EVENT_TABLE.lock();
        if let Some(_x) = &lock[index] {
            lock[index] = None;
            INTERRUPT_DISTRIBUTOR.lock().disable_interrupt(index as u32);
        } else {
            return ResultCode::new(Module::Kernel, Reason::Unknown);
        }
    }

    ResultCode::new(Module::Kernel, Reason::InvalidHandle)
}
