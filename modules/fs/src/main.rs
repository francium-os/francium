use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::{IPCServer, ServerImpl};
use process::os_error::{Module, OSError, OSResult, Reason};
use process::syscalls;
use process::Handle;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

struct FSServerStruct {}

impl FSServerStruct {
    fn test(&self) -> OSResult<TranslateMoveHandle> {
        Err(OSError::new(Module::FS, Reason::NotImplemented))
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from fs!");

    let port = syscalls::create_port("").unwrap();

    sm::register_port(syscalls::make_tag("fs"), TranslateCopyHandle(port)).unwrap();

    let server = Box::new(ServerImpl::new(FSServerStruct {}, port));

    server.process_forever().await;

    syscalls::close_handle(port).unwrap();
    println!("FS exiting!");

    syscalls::exit_process();
}
