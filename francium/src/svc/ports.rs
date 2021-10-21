use spin::Mutex;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::handle::HandleObject;
use crate::aarch64::context::ExceptionContext;

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

pub struct ClientPort {
	tag: u64
}

lazy_static! {
	static ref PORT_LIST: Mutex<Vec<HandleObject<ServerPort>>> = Mutex::new(Vec::new());
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
	// How: current process reference?

	ctx.regs[0] = 0;
	ctx.regs[1] = 0;
}

// request:
// x0 contains port name directly (as an 8 byte tag)
// response:
// x0 contains result
// x1 contains port handle (on success)
pub fn svc_connect_to_port(ctx: &mut ExceptionContext) {
	ctx.regs[0] = 0;
	ctx.regs[1] = 0;
}
