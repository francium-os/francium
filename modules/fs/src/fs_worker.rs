use std::sync::mpsc;
use process::os_error::OSResult;
use crate::block_adapter::BlockAdapter;

pub struct FSWorkerClient {
    request: mpsc::Sender<FSWorkerRequest>,
    response: mpsc::Receiver<FSWorkerResponse>,
}

impl FSWorkerClient {
    pub fn new(request: mpsc::Sender<FSWorkerRequest>, response: mpsc::Receiver<FSWorkerResponse>) -> FSWorkerClient {
        FSWorkerClient { request, response }
    }

    pub fn do_request(&self, request: FSWorkerRequest) -> OSResult<FSWorkerResponse> {
        self.request.send(request)?;
        Ok(self.response.recv()?)
    }
}

#[derive(Debug)]
pub enum FSWorkerRequest {
    Open(String),
    Read(usize),
    Write(usize),
    /* ... */
}

#[derive(Debug)]
pub enum FSWorkerResponse {
    /* An internal handle to the new file */
    Open(OSResult<usize>),
}


type FatFilesystem = fatfs::FileSystem<
    fatfs::StdIoWrapper<BlockAdapter>,
    fatfs::DefaultTimeProvider,
    fatfs::LossyOemCpConverter,
>;
pub fn fs_worker_thread(request: mpsc::Receiver<FSWorkerRequest>, response: mpsc::Sender<FSWorkerResponse>, fs: FatFilesystem) {
    println!("Hello from fs worker");
    loop {
        let req = request.recv().unwrap();
        match req {
            FSWorkerRequest::Open(filename) => {
                response.send(FSWorkerResponse::Open(Ok(0))).unwrap();
            }
            _ => {
                println!("AAAAAAAAAAA");
                panic!("Unknown FS worker request {:?}", req);
            }
        }
    }
}