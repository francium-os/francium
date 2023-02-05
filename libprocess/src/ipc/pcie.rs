use common::Handle;
use spin::Mutex;


#[derive(Copy, Clone, Default, Debug)]
pub struct PCIDeviceInfo {
    pub bus: u8,
    pub device: u8,
    pub vendor_id: u16,
    pub device_id: u16
}

use common::os_error::OSResult;
//use common::ipc::TranslateMoveHandle;
use crate::ipc::message::IPCMessage;
use crate::ipc::message::IPCValue;
impl IPCValue for PCIDeviceInfo {
    fn read(msg: &mut IPCMessage) -> PCIDeviceInfo {
        PCIDeviceInfo {
            bus: u8::read(msg),
            device: u8::read(msg),
            vendor_id: u16::read(msg),
            device_id: u16::read(msg)
        }
    }

    fn write(msg: &mut IPCMessage, val: &PCIDeviceInfo) {
        u8::write(msg, &val.bus);
        u8::write(msg, &val.device);

        u16::write(msg, &val.vendor_id);
        u16::write(msg, &val.device_id);
    }
}

static PCIE_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

fn get_handle_for_pcie() -> Handle {
    let mut locked = PCIE_HANDLE.lock();
    match *locked {
        Some(x) => x,
        None => {
            let handle = crate::ipc::sm::get_service_handle(crate::syscalls::make_tag("pcie"))
                .unwrap()
                .0;
            *locked = Some(handle);
            handle
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/pcie_client_impl.rs"));
