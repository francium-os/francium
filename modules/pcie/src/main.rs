use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::ServerImpl;
use process::syscalls;
use std::convert::TryInto;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

mod ecam;
#[cfg(target_arch = "x86_64")]
mod pcie_acpi;
#[cfg(target_arch = "aarch64")]
mod pcie_dt;
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

    fn get_devices_by_vidpid(&self, vid: u16, pid: u16) -> Vec<u32> {
        let mut devices = Vec::new();

        let buses_locked = self.buses.lock().unwrap();
        // this sucks but only a little
        for bus in buses_locked.iter() {
            for dev in bus.devices.iter() {
                for func in dev.functions.iter() {
                    // etc
                    if func.inner.header.vendor_id == vid && func.inner.header.device_id == pid {
                        devices.push(
                            bus.num as u32 * 256 * 256 + dev.num as u32 * 256 + func.num as u32,
                        );
                    }
                }
            }
        }

        devices
    }

    fn get_devices_by_class(&self, class: u8, subclass: u8) -> Vec<u32> {
        let mut devices = Vec::new();

        let buses_locked = self.buses.lock().unwrap();

        for bus in buses_locked.iter() {
            for dev in bus.devices.iter() {
                for func in dev.functions.iter() {
                    if func.inner.header.class == class && func.inner.header.subclass == subclass {
                        devices.push(
                            bus.num as u32 * 256 * 256 + dev.num as u32 * 256 + func.num as u32,
                        );
                    }
                }
            }
        }

        devices
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

    fn get_cap(&self, device: u32, cap_index: u8) -> OSResult<Vec<u8>> {
        // just kidding this sucks a lot
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let mut buses_locked = self.buses.lock().unwrap();

        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        // TODO: magic number!!!!!!
                        if (func.inner.header.status & (1<<4)) == (1<<4) {
                            // caps are supported
                            let cap_offset = func.inner.capabilities;
                            unsafe {
                                // TODO: explicit config space rework
                                let config_space_ptr = func.inner as *const ecam::ConfigurationSpaceType0 as *const u8;

                                let mut cap_ptr = config_space_ptr.add(cap_offset as usize);
                                let mut curr_cap_index = 0;

                                loop {
                                    let next_cap_offset = *cap_ptr.add(1);

                                    let cap_type = *cap_ptr;
                                    let cap_len = match cap_type {
                                        9 => *cap_ptr.add(2),
                                        0x11 => 0xc,
                                        _ => panic!("Unknown cap type 0x{:x}", cap_type)
                                    };

                                    if cap_index == curr_cap_index {
                                        /* TODO: uhhh
                                            virtio spec:
                                            For device configuration access, the driver MUST use 8-bit wide accesses for 8-bit wide fields, 16-bit wide
                                            and aligned accesses for 16-bit wide fields and 32-bit wide and aligned accesses for 32-bit and 64-bit wide
                                            fields. For 64-bit fields, the driver MAY access each of the high and low 32-bit parts of the field independently.
                                        */

                                        let cap: Vec<u8> = std::slice::from_raw_parts(cap_ptr, cap_len as usize).to_vec();
                                        return Ok(cap)
                                    }

                                    if next_cap_offset == 0 {
                                        break
                                    }

                                    curr_cap_index += 1;
                                    cap_ptr = config_space_ptr.add(next_cap_offset as usize);
                                }

                                // TODO: better return code
                                return Err(OSError::new(Module::PCIE, Reason::NotFound))
                            }
                        } else {
                            println!("Caps not supported");
                        }
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
    let pcie_buses = pcie_acpi::scan_via_acpi();
    #[cfg(target_arch = "x86_64")]
    let pci_32bit_addr = Some(0);
    #[cfg(target_arch = "x86_64")]
    let io_space_addr = Some(0);

    #[cfg(target_arch = "aarch64")]
    let (pcie_buses, io_space_addr, pci_32bit_addr, _pci_64bit_addr) = pcie_dt::scan_via_device_tree(0x40000000);

    let port = syscalls::create_port("").unwrap();
    sm::register_port(syscalls::make_tag("pcie"), TranslateCopyHandle(port)).unwrap();

    let server = ServerImpl::new(
        PCIEServerStruct {
            buses: Mutex::new(pcie_buses),
            mem_base: AtomicU32::new(pci_32bit_addr.unwrap().try_into().unwrap()),
            io_base: AtomicU32::new(io_space_addr.unwrap().try_into().unwrap()),
        },
        port,
    );

    server.process_forever().await;

    syscalls::exit_process();
}
