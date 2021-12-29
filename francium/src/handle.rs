use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::{Mutex};

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