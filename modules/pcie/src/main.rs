use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::ServerImpl;
use process::syscalls;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

mod ecam;
mod pcie;

use common::os_error::*;
use common::Handle;
use process::ipc::pcie::PCIDeviceInfo;
use process::ipc_server::IPCServer;
include!(concat!(env!("OUT_DIR"), "/pcie_server_impl.rs"));

struct PCIEServerStruct {
    buses: Mutex<Vec<pcie::PCIBus>>,
    mem_base: AtomicU32,
    io_base: AtomicU32,
}

impl PCIEServerStruct {
    fn list_devices(&self) -> Vec<PCIDeviceInfo> {
        let mut all_devices = Vec::new();

        let buses_locked = self.buses.lock().unwrap();

        for bus in buses_locked.iter() {
            for dev in bus.devices.iter() {
                for func in dev.functions.iter() {
                    all_devices.push(PCIDeviceInfo {
                        bus: bus.num,
                        device: dev.num,
                        vendor_id: func.inner.header.vendor_id,
                        device_id: func.inner.header.device_id,
                    });
                }
            }
        }

        all_devices
    }

    fn get_device_by_vidpid(&self, vid: u16, pid: u16) -> Option<u32> {
        let buses_locked = self.buses.lock().unwrap();
        // this sucks but only a little
        for bus in buses_locked.iter() {
            for dev in bus.devices.iter() {
                for func in dev.functions.iter() {
                    // etc
                    if func.inner.header.vendor_id == vid && func.inner.header.device_id == pid {
                        return Some(
                            bus.num as u32 * 256 * 256 + dev.num as u32 * 256 + func.num as u32,
                        );
                    }
                }
            }
        }

        None
    }

    fn enable(&self, device: u32) -> OSResult<()> {
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let mut buses_locked = self.buses.lock().unwrap();

        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        func.inner.header.command = 1 | 2;
                        return Ok(());
                    }
                }
            }
        }

        Err(OSError::new(Module::PCIE, Reason::NotFound))
    }

    fn get_bar(&self, device: u32, bar_index: u8) -> OSResult<(usize, usize)> {
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let mut buses_locked = self.buses.lock().unwrap();

        // just kidding this sucks a lot
        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    // etc
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        // Okay, how big is the BAR?

                        // TODO: Should probably be {read,write}_volatile to ptrs. Probably.
                        let bar_index = bar_index as usize;
                        let old_bar = func.inner.bars[bar_index];
                        let bar_type = old_bar & 0x1;
                        let old_bar_addr = old_bar & !0xf;

                        // TODO: 64bit bars...
                        func.inner.bars[bar_index] = 0xffffffff;
                        let bar_largest = func.inner.bars[bar_index] & !0xf;
                        let bar_size: usize = 0xffffffff - bar_largest as usize + 1;

                        // Ok, put the BAR back? Or make up our own BAR allocation. Augh.
                        let bar_base = if old_bar_addr == 0 {
                            let bar_base = if bar_type == 0 {
                                self.mem_base.fetch_add(bar_size as u32, Ordering::Acquire)
                            } else {
                                self.io_base.fetch_add(bar_size as u32, Ordering::Acquire)
                            };
                            func.inner.bars[bar_index] = bar_base;
                            bar_base
                        } else {
                            func.inner.bars[bar_index] = old_bar_addr;
                            old_bar_addr
                        };

                        return Ok((bar_base as usize, bar_size));
                    }
                }
            }
        }

        Err(OSError::new(Module::PCIE, Reason::NotFound))
    }
}

#[tokio::main]
async fn main() {
    println!("Hello from pcie!");

    #[cfg(target_arch = "x86_64")]
    let pcie_buses = pcie::scan_via_acpi();

    #[cfg(target_arch = "aarch64")]
    let pcie_buses = pcie::scan_via_device_tree(0x40000000); /*  [VIRT_PCIE_ECAM] =          { 0x3f000000, 0x01000000 }, */

    let port = syscalls::create_port("").unwrap();
    sm::register_port(syscalls::make_tag("pcie"), TranslateCopyHandle(port)).unwrap();

    let server = ServerImpl::new(
        PCIEServerStruct {
            buses: Mutex::new(pcie_buses),
            mem_base: AtomicU32::new(0x10000000),
            io_base: AtomicU32::new(0x3eff0000),
        },
        port,
    );

    server.process_forever().await;

    syscalls::exit_process();
}
