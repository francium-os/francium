use spin::Mutex;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::handle::HandleObject;
use crate::aarch64::context::ExceptionContext;
use crate::scheduler;

use alloc::sync::Arc;
use crate::process::Process;

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
	ports.push(HandleObject::new(Box::new(ServerPort::new(tag))));

	let mut port_waiters = PORT_WAITERS.lock();

	port_waiters.retain( |x| {
		let delete = {
			scheduler::wake_process(x.1.clone(), ctx);
			x.0 == tag
		};
		!delete
	});

	println!("{:?}", port_waiters);

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

	let mut ports = PORT_LIST.lock();
	for p in ports.iter() {
		if p.obj.lock().tag == tag {
			// Found the port we wanted.
			ctx.regs[0] = 0;
			ctx.regs[1] = 0;
			return
		}
	}

	// if we get here, the port isn't here yet.
	println!("Sleeping as we don't have the port yet. {:x}", tag);
	PORT_WAITERS.lock().push((tag, scheduler::get_current_process()));
	scheduler::suspend_current_process(ctx);

	ctx.regs[0] = 0;
	ctx.regs[1] = 0;
}
