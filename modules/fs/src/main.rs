use block_adapter::BlockAdapter;
use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::{IPCServer, ServerImpl};
use process::os_error::{Module, OSError, OSResult, Reason};
use process::syscalls;
use process::{Handle, INVALID_HANDLE};
use std::sync::Arc;
use std::sync::Mutex;

mod virtio_pci;

mod block;
mod block_adapter;
mod block_virtio;

use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

type FatFilesystem = fatfs::FileSystem<
    fatfs::StdIoWrapper<BlockAdapter>,
    fatfs::DefaultTimeProvider,
    fatfs::LossyOemCpConverter,
>;

struct FSServerStruct {
    // todo: hold multiple filesystems and implement some VFS stuff
    fs: Arc<Mutex<FatFilesystem>>,
}

fn map_fatfs_error(e: fatfs::Error<std::io::Error>) -> OSError {
    match e {
        fatfs::Error::NotFound => OSError::new(Module::Fs, Reason::NotFound),
        _ => OSError::new(Module::Fs, Reason::Unknown),
    }
}

impl FSServerStruct {
    fn open_file(&self, file_name: String) -> OSResult<TranslateMoveHandle> {
        println!("Hi from open_file!");

        let fs = self.fs.lock().unwrap();
        let mut file = fs
            .root_dir()
            .open_file(&file_name)
            .map_err(map_fatfs_error)?;
        let mut v: Vec<u8> = Vec::new();

        let starting_tick = syscalls::get_system_tick();
        file.read_to_end(&mut v).unwrap();
        let ending_tick = syscalls::get_system_tick();
        println!(
            "file len: {:?} in {} sec",
            v.len(),
            (ending_tick - starting_tick) as f64 / 1e9
        );

        let server_session: Handle = INVALID_HANDLE;
        let client_session: Handle = INVALID_HANDLE;
        let (server_session, client_session) = syscalls::create_session().unwrap();

        // todo: store server session _somewhere_
        println!("got file handle {:?}", client_session);
        Ok(TranslateMoveHandle(client_session))
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from fs!");

    unsafe {
        let mut ver_string_ptr: *const i8 = core::ptr::null();
        let mut date_string_ptr: *const i8 = core::ptr::null();
        let ret = ext2fs_sys::ext2fs_get_library_version(&mut ver_string_ptr, &mut date_string_ptr);

        println!("{} {:?} {:?}", ret, ver_string_ptr, core::ffi::CStr::from_ptr(ver_string_ptr));
    }

    let port = syscalls::create_port("").unwrap();

    sm::register_port(syscalls::make_tag("fs"), TranslateCopyHandle(port)).unwrap();

    let mut blocks = block_virtio::scan();
    let first_block = blocks.pop().unwrap();

    let adapted = Box::new(BlockAdapter::new(first_block.clone(), 0));

    let cfg = gpt::GptConfig::new().writable(false);
    let gpt_disk = cfg.open_from_device(adapted).unwrap();

    println!("Got partition table: {:?}", gpt_disk.partitions());

    let first_partition = gpt_disk.partitions().get(&1).unwrap();
    let partition_start = first_partition.first_lba;

    let adapted_partition = BlockAdapter::new(first_block, partition_start);

    let fs = fatfs::FileSystem::new(
        fatfs::StdIoWrapper::new(adapted_partition),
        fatfs::FsOptions::new(),
    )
    .unwrap();
    let first_fs = fs;

    let server = Box::new(ServerImpl::new(
        FSServerStruct {
            fs: Arc::new(Mutex::new(first_fs)),
        },
        port,
    ));

    println!("fs: processing");
    server.process_forever().await;

    syscalls::close_handle(port).unwrap();
    println!("FS exiting!");

    syscalls::exit_process();
}
