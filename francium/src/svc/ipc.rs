use crate::scheduler;
use crate::handle;
use crate::handle::HandleObject;
use crate::process::Thread;
use crate::waitable;
use crate::waitable::{Waiter, Waitable};
use common::os_error::{ResultCode, RESULT_OK, Module, Reason};
use common::ipc::*;
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::sync::{Arc, Weak};

use smallvec::SmallVec;

#[derive(Debug)]
pub struct ServerSession {
	wait: Waiter,
	connect_wait: Waiter,
	queue: Mutex<SmallVec<[Arc<Thread>; 1]>>,
	client: Mutex<Weak<ClientSession>>,
	client_thread: Mutex<Option<Arc<Thread>>>
}

#[derive(Debug)]
pub struct ClientSession {
	wait: Waiter,
	server: Arc<ServerSession>
}

#[derive(Debug)]
pub struct Port {
	wait: Waiter,
	// todo: queue default length
	queue: Mutex<SmallVec<[Arc<ServerSession>; 1]>>
}

impl Port {
	fn new() -> Port {
		Port {
			wait: Waiter::new(),
			queue: Mutex::new(SmallVec::new())
		}
	}
}

impl Waitable for Port { fn get_waiter(&self) -> &Waiter { &self.wait } }

impl ServerSession {
	fn new() -> ServerSession {
		ServerSession {
			wait: Waiter::new(),
			connect_wait: Waiter::new(),
			queue: Mutex::new(SmallVec::new()),
			client: Mutex::new(Weak::new()),
			client_thread: Mutex::new(None)
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

pub fn svc_create_port(tag: u64) -> (ResultCode, u32) {
	let server_port = Port::new();
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

	let handle_value = process.handle_table.get_handle(HandleObject::Port(server_port_handle));
	(RESULT_OK, handle_value)
}

pub fn svc_connect_to_port(tag: u64) -> (ResultCode, u32) {
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

	let server_session = Arc::new(ServerSession::new());
	let client_session = Arc::new(ClientSession::new(server_session.clone()));

	// TODO: ugh, i really wanted OnceCell here
	*server_session.client.lock() = Arc::downgrade(&client_session);

	// create the session, and wait for it to be accepted by the server
	port.queue.lock().push(server_session.clone());
	port.signal_one();
	server_session.connect_wait.wait();

	// return session
	{
		let current_process = scheduler::get_current_process();
		let mut process = current_process.lock();
		let handle_value = process.handle_table.get_handle(HandleObject::ClientSession(client_session));
		(RESULT_OK, handle_value)
	}
}

// x0: ipc session
pub fn svc_ipc_request(session_handle: u32) -> ResultCode {
	if let HandleObject::ClientSession(client_session) = handle::get_handle(session_handle) {
		// signal, then wait for reply
		let current_thread = scheduler::get_current_thread();

		client_session.server.queue.lock().push(current_thread);
		client_session.server.signal_one();
		client_session.wait();

		RESULT_OK
	} else {
		// error
		ResultCode::new(Module::Kernel, Reason::InvalidHandle)
	}
}

use crate::process::TLS_TCB_OFFSET;
const MAX_HANDLES: usize = 128;
pub fn svc_ipc_receive(handles_ptr: *const u32, handle_count: usize) -> (ResultCode, usize) {
	let mut handles: [u32; MAX_HANDLES] = [ 0xffffffff ; MAX_HANDLES];

	unsafe {
		core::ptr::copy_nonoverlapping(handles_ptr, &mut handles as *mut u32, handle_count);
	}

	let index = waitable::wait_handles(&handles[..handle_count]);

	if let HandleObject::ServerSession(server_session) = handle::get_handle(handles[index]) {
		let client_thread = server_session.queue.lock().pop().unwrap();
		let current_thread = scheduler::get_current_thread();
		/* Process IPC message here! */
		{
			let from_tls = client_thread.thread_local.lock();
			let mut to_tls = current_thread.thread_local.lock();

			unsafe {
				core::ptr::copy_nonoverlapping(from_tls[TLS_TCB_OFFSET..].as_ptr(), to_tls[TLS_TCB_OFFSET..].as_mut_ptr(), 128);
			}
		}

		*server_session.client_thread.lock() = Some(client_thread);
	}

	(RESULT_OK, index)
}

// x0: session handle
pub fn svc_ipc_reply(session_handle: u32) -> ResultCode {
	if let HandleObject::ServerSession(server_session) = handle::get_handle(session_handle) {
		// TODO: wtf?
		let current_thread = scheduler::get_current_thread();
		let mut thread_lock = server_session.client_thread.lock();
		let client_thread = thread_lock.as_ref().unwrap();

		{
			let from_tls = current_thread.thread_local.lock();
			let mut to_tls = client_thread.thread_local.lock();
			unsafe {
				core::ptr::copy_nonoverlapping(from_tls[TLS_TCB_OFFSET..].as_ptr(), to_tls[TLS_TCB_OFFSET..].as_mut_ptr(), 128);
			}
		}

		*thread_lock = None;
		server_session.client.lock().upgrade().unwrap().signal_one();
		RESULT_OK
	} else {
		ResultCode::new(Module::Kernel, Reason::InvalidHandle)
	}
}

// x0: port
// x1: session handle out
pub fn svc_ipc_accept(port_handle: u32) -> (ResultCode, u32) {
	if let HandleObject::Port(port) = handle::get_handle(port_handle) {
		let server_session = port.queue.lock().pop().unwrap();

		// wake the client
		server_session.connect_wait.signal_one();

		let current_process = scheduler::get_current_process();
		let mut process = current_process.lock();
		let handle_value = process.handle_table.get_handle(HandleObject::ServerSession(server_session));
		(RESULT_OK, handle_value)
	} else {
		(ResultCode::new(Module::Kernel, Reason::InvalidHandle), 0xffffffff)
	}
}
