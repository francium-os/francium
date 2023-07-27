use block_adapter::BlockAdapter;
use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::{IPCServer, ServerImpl};
use process::os_error::{Module, OSError, OSResult, Reason};
use process::syscalls;
use process::{define_server, define_session};
use process::Handle;
use std::sync::Arc;
use std::sync::Mutex;

mod virtio_pci;

mod block;
mod block_adapter;
mod block_virtio;

use std::sync::mpsc;
use std::thread;

include!(concat!(env!("OUT_DIR"), "/fs_server_impl.rs"));

struct ChannelPair {
    request: mpsc::Sender<()>,
    response: mpsc::Receiver<()>,
}
define_server! {
    FSServerStruct {
        fs_worker: Mutex<ChannelPair>
    }
}

define_session! {
    FSSession {},
    FSServerStruct
}

define_session! {
    IFileSession {},
    FSServerStruct
}

define_session! {
    IDirectorySession {},
    FSServerStruct
}

type FatFilesystem = fatfs::FileSystem<
    fatfs::StdIoWrapper<BlockAdapter>,
    fatfs::DefaultTimeProvider,
    fatfs::LossyOemCpConverter,
>;

fn map_fatfs_error(e: fatfs::Error<std::io::Error>) -> OSError {
    match e {
        fatfs::Error::NotFound => OSError::new(Module::Fs, Reason::NotFound),
        _ => OSError::new(Module::Fs, Reason::Unknown),
    }
}

impl FSServerStruct {
    fn accept_main_session(self: &Arc<FSServerStruct>) -> Arc<FSSession> {
        Arc::new(FSSession {
            __server: self.clone(),
        })
    }
}

impl FSSession {
    fn open_file(&self, file_name: String) -> OSResult<TranslateMoveHandle> {
        println!("Hi from open_file!");

        let server = self.get_server();
        let fs = server.fs_worker.lock();
        /*
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

        server.get_server_impl().register_session(
            server_session,
            Arc::new(IFileSession {
                __server: server.clone(),
            }),
        );*/

        let (server_session, client_session) = syscalls::create_session().unwrap();
        server.get_server_impl().register_session(
            server_session,
            Arc::new(IFileSession {
                __server: server.clone(),
            }),
        );

        println!("got file handle {:?}", client_session);
        Ok(TranslateMoveHandle(client_session))
    }
}

impl IFileSession {
    fn read_file(&self, length: usize) -> OSResult<usize> {
        println!("Read file");
        Ok(0)
    }
}

fn fs_worker_thread(request: mpsc::Receiver<()>, response: mpsc::Sender<()>, fs: FatFilesystem) {
    println!("Hello from fs worker");
    loop {}
}

#[tokio::main]
async fn main() {
    println!("Hello from fs!");

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

    let (tx_request, rx_request) = mpsc::channel();
    let (tx_response, rx_response) = mpsc::channel();

    let fs_worker_thread =
        thread::spawn(move || fs_worker_thread(rx_request, tx_response, first_fs));

    let server = Arc::new(FSServerStruct {
        __server_impl: Mutex::new(ServerImpl::new(port)),
        fs_worker: Mutex::new(ChannelPair {
            request: tx_request,
            response: rx_response,
        }),
    });

    println!("fs: processing");
    server.process_forever();

    tokio::task::block_in_place(|| fs_worker_thread.join().unwrap());

    syscalls::close_handle(port).unwrap();
    println!("FS exiting!");

    syscalls::exit_process();
}
