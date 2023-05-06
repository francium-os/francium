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

use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

type FatFilesystem = fatfs::FileSystem<fatfs::StdIoWrapper<BlockAdapter>, fatfs::DefaultTimeProvider, fatfs::LossyOemCpConverter>;

struct FSServerStruct {
    // todo: hold multiple filesystems and implement some VFS stuff
    fs: Arc<Mutex<FatFilesystem>>
}

impl FSServerStruct {
    fn open_file(&self, file_name: &str) -> OSResult<u32> {
        println!("Hi from open_file!");

        let fs = self.fs.lock().unwrap();
        
        let mut file = fs.root_dir().open_file(file_name).unwrap();
    	let mut s: String = String::new();
    	file.read_to_string(&mut s);
    	println!("{:?}", s);

    	Ok(0)
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from fs!");

    let port = syscalls::create_port("").unwrap();

    sm::register_port(syscalls::make_tag("fs"), TranslateCopyHandle(port)).unwrap();

    let mut blocks = block_virtio::scan();
    let first_block = blocks.pop().unwrap();
    // TODO: uhhhh, we need to parse the partition out of the {gpt, mbr}
    let adapted = BlockAdapter::new(first_block, 34);
    let fs = fatfs::FileSystem::new(fatfs::StdIoWrapper::new(adapted), fatfs::FsOptions::new()).unwrap();
    let first_fs = fs;

    let server = Box::new(ServerImpl::new(FSServerStruct {
        fs: Arc::new(Mutex::new(first_fs))
    }, port));

    println!("fs: processing");
    server.process_forever().await;

    syscalls::close_handle(port).unwrap();
    println!("FS exiting!");

    syscalls::exit_process();
}
