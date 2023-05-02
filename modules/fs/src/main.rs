use block_adapter::BlockAdapter;
use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::{IPCServer, ServerImpl};
use process::os_error::{Module, OSError, OSResult, Reason};
use process::syscalls;
use process::Handle;
use std::sync::Mutex;
use std::sync::Arc;

mod virtio_pci;

mod block;
mod block_adapter;
mod block_virtio;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

struct FSServerStruct<'a> {
    // todo: hold multiple filesystems and implement some VFS stuff
    fs: Arc<Mutex<fatfs::FileSystem<BlockAdapter<'a>>>>
}

impl FSServerStruct<'_> {
    fn open_file(&self, file_name: &str) -> OSResult<u32> {
        let fs = self.fs.lock().unwrap();
        
        fs.root_dir().open_file(file_name).ok_or(Err(OSError::new(Module::FS, Reason::NotFound)))
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from fs!");

    let port = syscalls::create_port("").unwrap();

    sm::register_port(syscalls::make_tag("fs"), TranslateCopyHandle(port)).unwrap();

    let server = Box::new(ServerImpl::new(FSServerStruct {}, port));

    let blocks = block_virtio::scan();
    let mut first_block = blocks.get(0);
    // TODO: uhhhh, we need to parse the partition out of the {gpt, mbr}
    let adapted = BlockAdapter::new(first_block, 34);
    let fs = fatfs::FileSystem::new(adapted, fatfs::FsOptions::new()).unwrap();
    let first_fs = fs;

    let server = Box::new(ServerImpl::new(FSServerStruct {
        fs: Arc::new(Mutex::new(first_fs))
    }, port));

    server.process_forever().await;

    syscalls::close_handle(port).unwrap();
    println!("FS exiting!");

    syscalls::exit_process();
}
