#![no_std]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
use process::println;
use process::syscalls;
use process::Handle;
use process::os_error::{OSError, OSResult, Module, Reason};
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::*;
use process::ipc::sm;
use process::ipc::fs::FSServer;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

struct FSServerStruct {
	should_stop: AtomicBool
}

impl FSServerStruct {
	fn stop(&self) {
		println!("FS stopping!");
		self.should_stop.store(true, Ordering::Release);
	}

	fn test(&self) -> OSResult<TranslateMoveHandle> {
		Err(OSError::new(Module::FS, Reason::NotImplemented))
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
