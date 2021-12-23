use spin::Mutex;
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
	fn new(serv: &ServerPort) -> ClientPort {
		ClientPort {
			tag: serv.tag
		}
	}
}

lazy_static! {
	static ref PORT_LIST: Mutex<Vec<HandleObject<ServerPort>>> = Mutex::new(Vec::new());
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
	for p in ports.iter() {
		if p.obj.lock().tag == tag {
			// Found already!
			panic!("Port already present!");
		}
	}

	let server_port = ServerPort::new(tag);

	let mut port_waiters = PORT_WAITERS.lock();
	port_waiters.retain( |x| {
		if x.0 == tag {
			let p = x.1.clone();
			{
				let mut process = p.lock();
				let client_port = Handle::ClientPort(HandleObject::new(Box::new(ClientPort::new(&server_port))));

				process.context.regs[0] = 0;
				let handle = process.handle_table.get_handle(client_port);
				process.context.regs[1] = handle as usize;
			}

			scheduler::wake_process(p);
			false
		} else {
			true
		}
	});

	let server_port_handle = HandleObject::new(Box::new(server_port));
	ports.push(server_port_handle);

	ctx.regs[0] = 0;
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
		for p in ports.iter() {
			if p.obj.lock().tag == tag {
				// Found the port we wanted.
				ctx.regs[0] = 0;
				ctx.regs[1] = 0;
				return
			}
		}
	}

	// if we get here, the port isn't here yet.
	PORT_WAITERS.lock().push((tag, scheduler::get_current_process()));
	scheduler::suspend_current_process();
	println!("Woke up after connect to port block!");
}
