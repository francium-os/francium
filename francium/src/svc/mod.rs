mod debug_output;
pub mod event;
mod exit_process;
mod futex;
mod get_system_info;
mod get_system_tick;
mod handle;
pub mod ipc;
mod memory;
mod process;
mod svc_break;
mod thread;
mod wait;

pub use debug_output::svc_debug_output;
pub use exit_process::svc_exit_process;
pub use handle::svc_close_handle;
pub use ipc::svc_connect_to_named_port;
pub use ipc::svc_connect_to_port_handle;
pub use ipc::svc_create_port;
pub use svc_break::svc_break;

pub use ipc::svc_ipc_accept;
pub use ipc::svc_ipc_receive;
pub use ipc::svc_ipc_reply;
pub use ipc::svc_ipc_request;

pub use memory::svc_map_device_memory;
pub use memory::svc_map_memory;
pub use memory::svc_query_physical_address;

pub use process::svc_create_thread;
pub use process::svc_get_process_id;
pub use process::svc_get_thread_id;

pub use thread::svc_sleep_ns;

pub use futex::svc_futex_wait;
pub use futex::svc_futex_wake;

pub use get_system_info::svc_get_system_info;

pub use get_system_tick::svc_get_system_tick;

pub use event::svc_bind_interrupt;
pub use event::svc_create_event;
pub use event::svc_unbind_interrupt;
pub use event::svc_signal_event;
pub use event::svc_clear_event;

pub use wait::svc_wait_one;
