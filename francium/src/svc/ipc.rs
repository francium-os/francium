use tracing::{event, Level};

use crate::handle;
use crate::handle::HandleObject;
use crate::mmu::phys_to_virt;
use crate::process::Thread;
use crate::scheduler;
use crate::waitable;
use crate::waitable::{Waitable, Waiter};
use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use common::ipc::*;
use common::os_error::{Module, Reason, ResultCode, RESULT_OK};
use common::Handle;
use core::convert::TryInto;
use spin::Mutex;

use smallvec::SmallVec;

#[derive(Debug)]
pub struct ServerSession {
    wait: Waiter,
    connect_wait: Waiter,
    pub queue: Mutex<SmallVec<[(Arc<Thread>, usize); 1]>>,
    client: Mutex<Weak<ClientSession>>,
    client_thread: Mutex<Option<(Arc<Thread>, usize)>>,
}

#[derive(Debug)]
pub struct ClientSession {
    wait: Waiter,
    server: Arc<ServerSession>,
}

#[derive(Debug)]
pub struct Port {
    wait: Waiter,
    // todo: queue default length
    pub queue: Mutex<SmallVec<[Arc<ServerSession>; 1]>>,
}

impl Port {
    fn new() -> Port {
        Port {
            wait: Waiter::new(),
            queue: Mutex::new(SmallVec::new()),
        }
    }
}

impl Waitable for Port {
    fn get_waiter(&self) -> &Waiter {
        &self.wait
    }
}

impl ServerSession {
    fn new() -> ServerSession {
        ServerSession {
            wait: Waiter::new(),
            connect_wait: Waiter::new(),
            queue: Mutex::new(SmallVec::new()),
            client: Mutex::new(Weak::new()),
            client_thread: Mutex::new(None),
        }
    }
}
impl Waitable for ServerSession {
    fn get_waiter(&self) -> &Waiter {
        &self.wait
    }
}

impl ClientSession {
    fn new(server: Arc<ServerSession>) -> ClientSession {
        ClientSession {
            wait: Waiter::new(),
            server: server,
        }
    }
}
impl Waitable for ClientSession {
    fn get_waiter(&self) -> &Waiter {
        &self.wait
    }
}

lazy_static! {
    static ref PORT_LIST: Mutex<BTreeMap<u64, Arc<Port>>> = Mutex::new(BTreeMap::new());
    static ref PORT_WAITERS: Mutex<Vec<(u64, Arc<Thread>)>> = Mutex::new(Vec::new());
}

pub fn svc_create_port(tag: u64) -> (ResultCode, u32) {
    event!(Level::TRACE, svc_name = "create_port", tag = tag);

    let server_port = Port::new();
    let server_port_handle = Arc::new(server_port);

    // if not a private port
    if tag != 0 {
        let mut ports = PORT_LIST.lock();
        if ports.contains_key(&tag) {
            panic!("panik");
        }

        let mut port_waiters = PORT_WAITERS.lock();
        port_waiters.retain(|x| {
            if x.0 == tag {
                scheduler::wake_thread(&x.1.clone(), 0xffffffffffffffff);
                false
            } else {
                true
            }
        });

        ports.insert(tag, server_port_handle.clone());
    }

    let proc_locked = scheduler::get_current_process();
    let mut process = proc_locked.lock();

    let handle_value = process
        .handle_table
        .get_handle(HandleObject::Port(server_port_handle));
    (RESULT_OK, handle_value)
}

fn connect_to_port_impl(port: &Arc<Port>) -> u32 {
    let server_session = Arc::new(ServerSession::new());
    let client_session = Arc::new(ClientSession::new(server_session.clone()));

    // TODO: ugh, i really wanted OnceCell here
    *server_session.client.lock() = Arc::downgrade(&client_session);

    // create the session, and wait for it to be accepted by the server

    port.queue.lock().push(server_session.clone());
    port.signal_one();
    server_session.connect_wait.wait();

    // return session
    {
        let current_process = scheduler::get_current_process();
        let mut process = current_process.lock();
        let handle_value = process
            .handle_table
            .get_handle(HandleObject::ClientSession(client_session));
        handle_value
    }
}

pub fn svc_connect_to_port_handle(h: u32) -> (ResultCode, u32) {
    event!(
        Level::TRACE,
        svc_name = "connect_to_port_handle",
        handle = h
    );

    if let HandleObject::Port(port) = handle::get_handle(h) {
        (RESULT_OK, connect_to_port_impl(&port))
    } else {
        (
            ResultCode::new(Module::Kernel, Reason::InvalidHandle),
            0xffffffff,
        )
    }
}

pub fn svc_connect_to_named_port(tag: u64) -> (ResultCode, u32) {
    event!(Level::TRACE, svc_name = "connect_to_named_port", tag = tag);

    let port = {
        let ports = PORT_LIST.lock();
        if let Some(server_port) = ports.get(&tag) {
            server_port.clone()
        } else {
            // make sure to drop the lock guard before suspending ourselves!
            drop(ports);

            PORT_WAITERS
                .lock()
                .push((tag, scheduler::get_current_thread()));
            scheduler::suspend_current_thread();

            // oops, try again
            {
                let ports = PORT_LIST.lock();
                ports.get(&tag).unwrap().clone()
            }
        }
    };
    (RESULT_OK, connect_to_port_impl(&port))
}

// x0: ipc session
pub fn svc_ipc_request(session_handle: u32, ipc_buffer_ptr: usize) -> ResultCode {
    event!(
        Level::TRACE,
        svc_name = "ipc_request",
        session_handle = session_handle,
        ipc_buffer_ptr = ipc_buffer_ptr
    );

    if let HandleObject::ClientSession(client_session) = handle::get_handle(session_handle) {
        // signal, then wait for reply
        let current_thread = scheduler::get_current_thread();

        client_session
            .server
            .queue
            .lock()
            .push((current_thread, ipc_buffer_ptr));
        client_session.server.signal_one();
        client_session.wait();

        RESULT_OK
    } else {
        // error
        ResultCode::new(Module::Kernel, Reason::InvalidHandle)
    }
}

const IPC_BUFFER_LEN: usize = 128;

fn do_ipc_transfer(
    from_thread: &Arc<Thread>,
    to_thread: &Arc<Thread>,
    from_ptr: usize,
    to_ptr: usize,
) {
    let from_ipc_buffer_ptr = phys_to_virt(
        from_thread
            .process
            .lock()
            .address_space
            .page_table
            .virt_to_phys(from_ptr)
            .unwrap(),
    ) as *const u8;
    let to_ipc_buffer_ptr = phys_to_virt(
        to_thread
            .process
            .lock()
            .address_space
            .page_table
            .virt_to_phys(to_ptr)
            .unwrap(),
    ) as *mut u8;

    unsafe {
        core::ptr::copy_nonoverlapping(from_ipc_buffer_ptr, to_ipc_buffer_ptr, 128);
    }

    let to_ipc_buffer =
        unsafe { core::slice::from_raw_parts_mut(to_ipc_buffer_ptr as *mut u8, IPC_BUFFER_LEN) };

    let packed_header = u32::from_le_bytes(to_ipc_buffer[0..4].try_into().unwrap());
    let header = IPCHeader::unpack(packed_header);

    // Translate all translate parameters
    for i in 0..header.translate_count {
        let off = header.size + i * 16;
        let entry = TranslateEntry::read(to_ipc_buffer[off..off + 16].try_into().unwrap());

        let new_entry = match entry {
            TranslateEntry::CopyHandle(handle) => {
                let obj = from_thread.process.lock().handle_table.get_object(handle.0);
                let new_handle = to_thread.process.lock().handle_table.get_handle(obj);
                TranslateEntry::CopyHandle(Handle(new_handle))
            }
            TranslateEntry::MoveHandle(handle) => {
                let obj = from_thread.process.lock().handle_table.get_object(handle.0);
                let new_handle = to_thread.process.lock().handle_table.get_handle(obj);
                assert!(from_thread.process.lock().handle_table.close(handle.0) == RESULT_OK);

                TranslateEntry::MoveHandle(Handle(new_handle))
            }
            _ => {
                unimplemented!("Can't translate {:?}", entry);
            }
        };
        TranslateEntry::write(&mut to_ipc_buffer[off..off + 16], new_entry);
    }
}

const MAX_HANDLES: usize = 128;
pub fn svc_ipc_receive(
    handles_ptr: *const u32,
    handle_count: usize,
    ipc_buffer_ptr: usize,
) -> (ResultCode, usize) {
    event!(
        Level::TRACE,
        svc_name = "ipc_receive",
        handles_ptr = handles_ptr as usize,
        handle_count = handle_count,
        ipc_buffer_ptr = ipc_buffer_ptr
    );

    let mut handles: [u32; MAX_HANDLES] = [0xffffffff; MAX_HANDLES];

    unsafe {
        core::ptr::copy_nonoverlapping(handles_ptr, &mut handles as *mut u32, handle_count);
    }

    let index = waitable::wait_handles(&handles[..handle_count]);

    if let HandleObject::ServerSession(server_session) = handle::get_handle(handles[index]) {
        let (client_thread, client_buffer_ptr) = server_session.queue.lock().pop().unwrap();
        let current_thread = scheduler::get_current_thread();

        // XX todo: figure out how to map from_ptr/to_ptr with respect to caches.
        do_ipc_transfer(
            &client_thread,
            &current_thread,
            client_buffer_ptr,
            ipc_buffer_ptr,
        );

        *server_session.client_thread.lock() = Some((client_thread, client_buffer_ptr));
    }

    (RESULT_OK, index)
}

// x0: session handle
pub fn svc_ipc_reply(session_handle: u32, ipc_buffer_ptr: usize) -> ResultCode {
    event!(
        Level::TRACE,
        svc_name = "ipc_reply",
        session_handle = session_handle,
        ipc_buffer_ptr = ipc_buffer_ptr
    );

    if let HandleObject::ServerSession(server_session) = handle::get_handle(session_handle) {
        // TODO: wtf?
        let current_thread = scheduler::get_current_thread();
        let mut thread_lock = server_session.client_thread.lock();
        let (client_thread, client_buffer_ptr) = thread_lock.as_ref().unwrap();

        // XX todo: figure out how to map from_ptr/to_ptr with respect to caches.
        do_ipc_transfer(
            &current_thread,
            &client_thread,
            ipc_buffer_ptr,
            *client_buffer_ptr,
        );

        *thread_lock = None;
        server_session.client.lock().upgrade().unwrap().signal_one();
        RESULT_OK
    } else {
        ResultCode::new(Module::Kernel, Reason::InvalidHandle)
    }
}

// x0: port
// x1: session handle out
pub fn svc_ipc_accept(port_handle: u32) -> (ResultCode, u32) {
    event!(
        Level::TRACE,
        svc_name = "ipc_accept",
        port_handle = port_handle
    );

    if let HandleObject::Port(port) = handle::get_handle(port_handle) {
        let server_session = port.queue.lock().pop().unwrap();

        // wake the client
        server_session.connect_wait.signal_one();

        let current_process = scheduler::get_current_process();
        let mut process = current_process.lock();
        let handle_value = process
            .handle_table
            .get_handle(HandleObject::ServerSession(server_session));
        (RESULT_OK, handle_value)
    } else {
        (
            ResultCode::new(Module::Kernel, Reason::InvalidHandle),
            0xffffffff,
        )
    }
}
