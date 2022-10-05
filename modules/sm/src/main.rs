#![no_std]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
use process::println;
use process::syscalls;
use process::{Handle, INVALID_HANDLE};
use process::os_error::{OSError, OSResult, Module, Error};
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::message::TranslateHandle;
use process::ipc::sm::SMServer;

struct SMServerStruct {
	should_stop: AtomicBool
}

impl IPCServer for SMServerStruct {
	fn process(&self, h: Handle) {
		SMServer::process(self, h)
	}
}

impl SMServer for SMServerStruct {
	fn stop(&self) {
		println!("SM stopping!");
		self.should_stop.store(true, Ordering::Release);
	}

	fn get_service_handle(&self, tag: u64) -> OSResult<TranslateHandle> {
		println!("Got tag: {:x}", tag);
		Ok(TranslateHandle(INVALID_HANDLE))
	}

	fn register_port(&self, tag: u64, port_handle: TranslateHandle) -> OSResult<()> {
		Err(OSError { module: Module::SM, err: Error::NotImplemented })
	}
}

fn main() {
	println!("Hello from sm!");

	let port = syscalls::create_port("sm").unwrap();
	let mut server = ServerImpl::new(SMServerStruct{ should_stop: AtomicBool::new(false) }, port);

	while server.process() {
		if server.server.should_stop.load(Ordering::Acquire) {
			break
		}
	}

	syscalls::close_handle(port).unwrap();
	println!("SM exiting!");

	syscalls::exit_process();
}
