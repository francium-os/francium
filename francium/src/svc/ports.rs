use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::handle::HandleObject;
use crate::aarch64::context::ExceptionContext;
use crate::scheduler;

use alloc::sync::Arc;
use crate::process::Process;
use crate::handle::Handle;

#[derive(Debug)]
pub struct ServerPort {
	tag: u64
}

impl ServerPort {
	fn new(tag: u64) -> ServerPort {
		ServerPort {
			tag: tag
		}
	}
}

#[derive(Debug)]
pub struct ClientPort {
	tag: u64
}

impl ClientPort {
	fn new(serv: &HandleObject<ServerPort>) -> ClientPort {
		ClientPort {
			tag: serv.lock().tag
		}
	}
}

lazy_static! {
	static ref PORT_LIST: Mutex<BTreeMap<u64, HandleObject<ServerPort>>> = Mutex::new(BTreeMap::new());
	static ref PORT_WAITERS: Mutex<Vec<(u64, Arc<Mutex<Box<Process>>>)>> = Mutex::new(Vec::new());
}

// request:
// x0 contains port name directly (as an 8 byte tag)
// response:
// x0 contains result
// x1 contains port handle (on success)
pub fn svc_create_port(ctx: &mut ExceptionContext) {
	let tag = ctx.regs[0] as u64;

	let mut ports = PORT_LIST.lock();
	if ports.contains_key(&tag) {
		panic!("panik");
	}
	
	let server_port = ServerPort::new(tag);

	let mut port_waiters = PORT_WAITERS.lock();
	port_waiters.retain( |x| {
		if x.0 == tag {
			scheduler::wake_process(x.1.clone());
			false
		} else {
			true
		}
	});

	let server_port_handle = HandleObject::new(Box::new(server_port));
	ports.insert(tag, server_port_handle.clone());

	let proc_locked = scheduler::get_current_process();
	let mut process = proc_locked.lock();

	ctx.regs[0] = process.handle_table.get_handle(Handle::ServerPort(server_port_handle)) as usize;
	ctx.regs[1] = 0;
}

// request:
// x0 contains port name directly (as an 8 byte tag)
// response:
// x0 contains result
// x1 contains port handle (on success)
pub fn svc_connect_to_port(ctx: &mut ExceptionContext) {
	let tag = ctx.regs[0] as u64;

	{
		let ports = PORT_LIST.lock();
		match ports.get(&tag) {
			Some(server_port) => {
				// todo: change this to use tpidr instead
				let proc_locked = scheduler::get_current_process();
				let mut process = proc_locked.lock();

				let client_port = Handle::ClientPort(HandleObject::new(Box::new(ClientPort::new(server_port))));
				let handle = process.handle_table.get_handle(client_port);
				ctx.regs[0] = 0;
				ctx.regs[1] = handle as usize;
			},
			None => {}
		}
	}

	// if we get here, the port isn't here yet.
	PORT_WAITERS.lock().push((tag, scheduler::get_current_process()));
	scheduler::suspend_current_process();

	// We know it must be in the ports list, otherwise why would we have been woken up?

	let ports = PORT_LIST.lock();
	match ports.get(&tag) {
		Some(server_port) => {
			// todo: change this to use tpidr instead
			let proc_locked = scheduler::get_current_process();
			let mut process = proc_locked.lock();

			let client_port = Handle::ClientPort(HandleObject::new(Box::new(ClientPort::new(server_port))));
			let handle = process.handle_table.get_handle(client_port);
			ctx.regs[0] = 0;
			ctx.regs[1] = handle as usize;
		},
		None => panic!("Port is still missing after wakeup?")
	}
}
