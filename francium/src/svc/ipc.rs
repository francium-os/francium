use crate::aarch64::context::ExceptionContext;
use crate::scheduler;
use crate::handle::{Handle};
use crate::process::Process;
use crate::waitable::{Waiter, Waitable};
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::sync::Arc;

use smallvec::SmallVec;

#[derive(Debug)]
pub struct ServerSession {
	wait: Waiter,
	port: Arc<Box<Port>>
}

#[derive(Debug)]
pub struct ClientSession {
	server: Arc<Box<ServerSession>>
}


#[derive(Debug)]
pub struct Port {
	wait: Waiter,
	tag: u64,
	// todo: queue default length
	queue: Mutex<SmallVec<[Arc<Box<ServerSession>>; 1]>>
}

impl Port {
	fn new(tag: u64) -> Port {
		Port {
			wait: Waiter::new(),
			tag: tag,
			queue: Mutex::new(SmallVec::new())
		}
	}
}

impl Waitable for Port { fn get_waiter(&self) -> &Waiter { &self.wait } }

impl ServerSession {
	fn new(port: Arc<Box<Port>>) -> ServerSession {
		ServerSession {
			wait: Waiter::new(),
			port: port
		}
	}
}
impl Waitable for ServerSession { fn get_waiter(&self) -> &Waiter { &self.wait } }

impl ClientSession {
	fn new(server: Arc<Box<ServerSession>>) -> ClientSession {
		ClientSession {
			server: server
		}
	}
}

lazy_static! {
	static ref PORT_LIST: Mutex<BTreeMap<u64, Arc<Box<Port>>>> = Mutex::new(BTreeMap::new());
	static ref PORT_WAITERS: Mutex<Vec<(u64, Arc<Mutex<Box<Process>>>)>> = Mutex::new(Vec::new());
}

// request:
// x0 contains port name directly (as an 8 byte tag)
// response:
// x0 contains result
// x1 contains port handle (on success)
pub fn svc_create_port(ctx: &mut ExceptionContext) {
	let tag = ctx.regs[0] as u64;

	let server_port = Port::new(tag);
	let server_port_handle = Arc::new(Box::new(server_port));

	// if not a private port
	if tag != 0 {
		let mut ports = PORT_LIST.lock();
		if ports.contains_key(&tag) {
			panic!("panik");
		}

		let mut port_waiters = PORT_WAITERS.lock();
		port_waiters.retain( |x| {
			if x.0 == tag {
				scheduler::wake_process(x.1.clone());
				false
			} else {
				true
			}
		});

		ports.insert(tag, server_port_handle.clone());
	}

	let proc_locked = scheduler::get_current_process();
	let mut process = proc_locked.lock();

	ctx.regs[0] = process.handle_table.get_handle(Handle::Port(server_port_handle)) as usize;
	ctx.regs[1] = 0;
}

fn svc_create_session() {

}

// request:
// x0 contains port name directly (as an 8 byte tag)
// response:
// x0 contains result
// x1 contains port handle (on success)
pub fn svc_connect_to_port(exc: &mut ExceptionContext) {
	let tag = exc.regs[0] as u64;

	let port = {
		let ports = PORT_LIST.lock();
		if let Some(server_port) = ports.get(&tag) {
			server_port.clone()
		} else {
			// make sure to drop the lock guard before suspending ourselves!
			drop(ports);

			PORT_WAITERS.lock().push((tag, scheduler::get_current_process()));
			scheduler::suspend_current_process();

			// oops, try again
			{
				let ports = PORT_LIST.lock();
				ports.get(&tag).unwrap().clone()
			}
		}
	};

	let server_session = Arc::new(Box::new(ServerSession::new(port.clone())));
	let client_session = Arc::new(Box::new(ClientSession::new(server_session.clone())));
	
	// create the session, and wait for it to be accepted by the server
	port.queue.lock().push(server_session.clone());
	port.signal_one();
	server_session.wait();

	// return session
	{
		let current_process = scheduler::get_current_process();
		let mut process = current_process.lock();
		exc.regs[0] = 0;
		exc.regs[1] = process.handle_table.get_handle(Handle::ClientSession(client_session)) as usize;
	}
}

// x0: ipc session
pub fn svc_ipc_request(exc: &mut ExceptionContext) {
	let ipc_session = {
		let process_locked = scheduler::get_current_process();
		let x = process_locked.lock().handle_table.get_object(exc.regs[0] as u32);
		x
	};

	if let Handle::ClientSession(session_handle) = ipc_session {
		// good

	} else {
		exc.regs[0] = 1;
		// error
	}
	unimplemented!();
}

// todo: setup tls??

// x0: ipc session
// x1: handles[]
// x2: handle_count

pub fn svc_ipc_receive(exc: &mut ExceptionContext) {
	let ipc_session = {
		let process_locked = scheduler::get_current_process();
		let x = process_locked.lock().handle_table.get_object(exc.regs[0] as u32);
		x
	};

	if let Handle::Port(port) = ipc_session {
		port.wait();

		if port.queue.lock().len() > 0 {
			exc.regs[0] = 0; // ok
			exc.regs[1] = 0; // signal the port
		} else {
			println!("queue = 0! what do i do now??");

			exc.regs[0] = 1; // error
			exc.regs[1] = 0;
		}
	} else {
		println!("non port handle to svc_ipc_receive: {:?}", ipc_session);
		exc.regs[0] = 1;
		// error
	}
}

// x0: session handle
// x1: ipc buffer
pub fn svc_ipc_reply(_exc: &mut ExceptionContext) {
	unimplemented!();
}

// x0: port
// x1: session handle out

pub fn svc_ipc_accept(exc: &mut ExceptionContext) {
	let port_handle = {
		let process_locked = scheduler::get_current_process();
		let x = process_locked.lock().handle_table.get_object(exc.regs[0] as u32);
		x
	};

	if let Handle::Port(port) = port_handle {
		let server_session = port.queue.lock().pop().unwrap();

		// wake the client
		server_session.signal_one();

		let current_process = scheduler::get_current_process();
		let mut process = current_process.lock();
		exc.regs[1] = process.handle_table.get_handle(Handle::ServerSession(server_session)) as usize;

		exc.regs[0] = 0;
		// todo handle
	} else {
		exc.regs[0] = 1;
		// error
	}
}