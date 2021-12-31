use crate::aarch64::context::ExceptionContext;
use crate::scheduler;
use crate::handle;
use crate::handle::Handle;
use crate::process::Thread;
use crate::waitable;
use crate::waitable::{Waiter, Waitable};
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};

use smallvec::SmallVec;

#[derive(Debug)]
pub struct ServerSession {
	wait: Waiter,
	port: Arc<Port>,
	client: Mutex<Weak<ClientSession>>
}

#[derive(Debug)]
pub struct ClientSession {
	wait: Waiter,
	server: Arc<ServerSession>
}

#[derive(Debug)]
pub struct Port {
	wait: Waiter,
	tag: u64,
	// todo: queue default length
	queue: Mutex<SmallVec<[Arc<ServerSession>; 1]>>
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
	fn new(port: Arc<Port>) -> ServerSession {
		ServerSession {
			wait: Waiter::new(),
			port: port,
			client: Mutex::new(Weak::new())
		}
	}
}
impl Waitable for ServerSession { fn get_waiter(&self) -> &Waiter { &self.wait } }

impl ClientSession {
	fn new(server: Arc<ServerSession>) -> ClientSession {
		ClientSession {
			wait: Waiter::new(),
			server: server
		}
	}
}
impl Waitable for ClientSession { fn get_waiter(&self) -> &Waiter { &self.wait } }

lazy_static! {
	static ref PORT_LIST: Mutex<BTreeMap<u64, Arc<Port>>> = Mutex::new(BTreeMap::new());
	static ref PORT_WAITERS: Mutex<Vec<(u64, Arc<Thread>)>> = Mutex::new(Vec::new());
}

// request:
// x0 contains port name directly (as an 8 byte tag)
// response:
// x0 contains result
// x1 contains port handle (on success)
pub fn svc_create_port(ctx: &mut ExceptionContext) {
	let tag = ctx.regs[0] as u64;

	let server_port = Port::new(tag);
	let server_port_handle = Arc::new(server_port);

	// if not a private port
	if tag != 0 {
		let mut ports = PORT_LIST.lock();
		if ports.contains_key(&tag) {
			panic!("panik");
		}

		let mut port_waiters = PORT_WAITERS.lock();
		port_waiters.retain( |x| {
			if x.0 == tag {
				scheduler::wake_thread(x.1.clone(), 0);
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

			PORT_WAITERS.lock().push((tag, scheduler::get_current_thread()));
			scheduler::suspend_current_thread();

			// oops, try again
			{
				let ports = PORT_LIST.lock();
				ports.get(&tag).unwrap().clone()
			}
		}
	};

	let server_session = Arc::new(ServerSession::new(port.clone()));
	let client_session = Arc::new(ClientSession::new(server_session.clone()));

	// TODO: ugh, i really wanted OnceCell here
	*server_session.client.lock() = Arc::downgrade(&client_session);
		
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
	if let Handle::ClientSession(client_session) = handle::get_handle(exc.regs[0]) {
		// signal, then wait for reply
		client_session.server.signal_one();
		client_session.wait();

		exc.regs[0] = 0;
	} else {
		// error
		exc.regs[0] = 1;
	}
}

// todo: setup tls??

// x0: ipc session
// x1: handles[]
// x2: handle_count

const MAX_HANDLES: usize = 128;

pub fn svc_ipc_receive(exc: &mut ExceptionContext) {
	if let Handle::Port(port) = handle::get_handle(exc.regs[0]) {
		let handle_count = exc.regs[2];
		let mut handles: [u32; MAX_HANDLES] = [ 0xffffffff ; MAX_HANDLES];

		unsafe {
			core::ptr::copy_nonoverlapping(exc.regs[1] as *const u32, &mut handles as *mut u32, handle_count);
		}

		let index = waitable::wait_handles(&handles[..handle_count]);

		exc.regs[0] = 0;
		exc.regs[1] = index;
		
	} else {
		println!("non port handle to svc_ipc_receive :(");
		exc.regs[0] = 1;
	}
}

// x0: session handle
pub fn svc_ipc_reply(exc: &mut ExceptionContext) {
	if let Handle::ServerSession(server_session) = handle::get_handle(exc.regs[0]) {
		exc.regs[0] = 0;

		// TODO: wtf?
		server_session.client.lock().upgrade().unwrap().signal_one();
	} else {
		exc.regs[0] = 1;
	}
}

// x0: port
// x1: session handle out

pub fn svc_ipc_accept(exc: &mut ExceptionContext) {
	if let Handle::Port(port) =  handle::get_handle(exc.regs[0]) {
		let server_session = port.queue.lock().pop().unwrap();

		// wake the client
		server_session.signal_one();

		let current_process = scheduler::get_current_process();
		let mut process = current_process.lock();
		exc.regs[1] = process.handle_table.get_handle(Handle::ServerSession(server_session)) as usize;
		exc.regs[0] = 0;
	} else {
		exc.regs[0] = 1;
		// error
	}
}