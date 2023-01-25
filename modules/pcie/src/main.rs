use process::syscalls;
use process::ipc_server::ServerImpl;
use process::ipc::sm;
use process::ipc::*;

mod ecam;
mod pcie;

use common::Handle;
use process::ipc_server::IPCServer;
use process::ipc::pcie::PCIDeviceInfo;
include!(concat!(env!("OUT_DIR"), "/pcie_server_impl.rs"));

struct PCIEServerStruct {
    buses: Vec<pcie::PCIBus>
}

impl PCIEServerStruct {
    fn list_devices(&self) -> Vec<PCIDeviceInfo> {
        let mut all_devices = Vec::new();

        for bus in self.buses.iter() {
            for dev in bus.devices.iter() {
                for func in dev.functions.iter() {
                    all_devices.push(PCIDeviceInfo {
                        bus: bus.num,
                        device: dev.num,
                        vendor_id: func.inner.vendor_id,
                        device_id: func.inner.device_id
                    });
                }
            }
        }

        all_devices
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from pcie!");

    let pcie_buses = pcie::scan_via_acpi();
    println!("Hello again from pcie");

    let port = syscalls::create_port("").unwrap();
    sm::register_port(syscalls::make_tag("pcie"), TranslateCopyHandle(port)).unwrap();

    let server = ServerImpl::new(
        PCIEServerStruct {
            buses: pcie_buses
        },
        port,
    );

    server.process_forever().await;

    syscalls::exit_process();
}
