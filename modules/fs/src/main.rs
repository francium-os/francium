use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::{IPCServer, ServerImpl};
use process::os_error::{Module, OSError, OSResult, Reason};
use process::syscalls;
use process::Handle;
use block_adapter::BlockAdapter;

mod virtio_pci;

mod block;
mod block_virtio;
mod block_adapter;

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

    let blocks = block_virtio::scan();
    for mut b in blocks {
        // TODO: uhhhh, we need to parse the partition out of the {gpt, mbr}

        let adapted = BlockAdapter::new(b.as_mut(), 34);
        let fs = fatfs::FileSystem::new(adapted, fatfs::FsOptions::new()).unwrap();

        let root_dir = fs.root_dir();
        for r in root_dir.iter() {
            let entry = r.unwrap();
            println!("file: {}", entry.file_name());
        }
    }

    server.process_forever().await;

    syscalls::close_handle(port).unwrap();
    println!("FS exiting!");

    syscalls::exit_process();
}
