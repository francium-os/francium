#![no_std]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
use process::println;
use process::syscalls;
use process::Handle;
use process::os_error::{OSError, OSResult, Module, Reason};
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::message::*;
use process::ipc::sm;
use process::ipc::fs::FSServer;

struct FSServerStruct {
	should_stop: AtomicBool
}

impl IPCServer for FSServerStruct {
	fn process(&self, h: Handle) {
		FSServer::process(self, h)
	}
}

impl FSServer for FSServerStruct {
	fn stop(&self) {
		println!("FS stopping!");
		self.should_stop.store(true, Ordering::Release);
	}

	fn test(&self) -> OSResult<TranslateMoveHandle> {
		Err(OSError { module: Module::FS, reason: Reason::NotImplemented })
	}
}

fn main() {
	println!("Hello from fs!");

	let port = syscalls::create_port("").unwrap();

	sm::register_port(syscalls::make_tag("fs"), TranslateCopyHandle(port)).unwrap();

	let mut server = ServerImpl::new(FSServerStruct{ should_stop: AtomicBool::new(false) }, port);

	while server.process() {
		if server.server.should_stop.load(Ordering::Acquire) {
			break
		}
	}

	syscalls::close_handle(port).unwrap();
	println!("FS exiting!");

	syscalls::exit_process();
}
