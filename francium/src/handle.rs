use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::{Mutex,MutexGuard};

use crate::process::Process;
use crate::memory::AddressSpace;
use crate::svc::ipc::{Port,ServerSession,ClientSession};
use crate::waitable::{Waiter, Waitable};

#[derive(Debug, Clone)]
pub enum Handle {
	Process(Arc<Mutex<Box<Process>>>),
	AddressSpace(Arc<Mutex<Box<AddressSpace>>>),
	Port(Arc<Box<Port>>),
	ServerSession(Arc<Box<ServerSession>>),
	ClientSession(Arc<Box<ClientSession>>),
	Invalid
}