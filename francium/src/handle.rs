use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;

use crate::memory::AddressSpace;
use crate::process::Process;
use crate::scheduler;
use crate::svc::event::Event;
use crate::svc::ipc::{ClientSession, Port, ServerSession};

#[derive(Debug, Clone)]
pub enum HandleObject {
    Process(Arc<Mutex<Box<Process>>>),
    AddressSpace(Arc<Mutex<Box<AddressSpace>>>),
    Port(Arc<Port>),
    ServerSession(Arc<ServerSession>),
    ClientSession(Arc<ClientSession>),
    Event(Arc<Event>),
    Invalid,
}

pub fn get_handle(reg: u32) -> HandleObject {
    let process_locked = scheduler::get_current_process();
    let x = process_locked.lock().handle_table.get_object(reg);
    x
}
