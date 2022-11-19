use process::syscalls;
use process::Handle;
use process::os_error::{OSError, OSResult, Module, Reason};
use process::ipc_server::{ServerImpl, IPCServer};
use process::ipc::*;
use process::ipc::sm;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

struct FSServerStruct {
}

impl FSServerStruct {
	fn stop(&self) {
		println!("TODO: Stop?");
	}

	fn test(&self) -> OSResult<TranslateMoveHandle> {
		Err(OSError::new(Module::FS, Reason::NotImplemented))
	}
}

#[tokio::main]
async fn main() {
	println!("Hello from fs!");

	let port = syscalls::create_port("").unwrap();

	sm::register_port(syscalls::make_tag("fs"), TranslateCopyHandle(port)).unwrap();

	let server = Box::new(ServerImpl::new(FSServerStruct{}, port));

	server.process_forever().await;

	syscalls::close_handle(port).unwrap();
	println!("FS exiting!");

	syscalls::exit_process();
}
