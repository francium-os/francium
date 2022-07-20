use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::{Mutex};

use crate::scheduler;
use crate::process::Process;
use crate::memory::AddressSpace;
use crate::svc::ipc::{Port,ServerSession,ClientSession};

#[derive(Debug, Clone)]
pub enum Handle {
	Process(Arc<Mutex<Box<Process>>>),
	AddressSpace(Arc<Mutex<Box<AddressSpace>>>),
	Port(Arc<Port>),
	ServerSession(Arc<ServerSession>),
	ClientSession(Arc<ClientSession>),
	Invalid
}

pub fn get_handle(reg: u32) -> Handle {
	let process_locked = scheduler::get_current_process();
	let x = process_locked.lock().handle_table.get_object(reg);
	x
}