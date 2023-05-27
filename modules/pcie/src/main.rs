use process::ipc::sm;
use process::ipc::*;
use process::ipc_server::ServerImpl;
use process::syscalls;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

mod ecam;
mod interrupt_map;
mod pcie;
#[cfg(target_arch = "x86_64")]
mod pcie_acpi;
#[cfg(target_arch = "aarch64")]
mod pcie_dt;

use common::Handle;
use process::ipc::pcie::PCIDeviceInfo;
use process::ipc_server::IPCServer;
use process::os_error::*;
use process::{define_server, define_session};

use crate::interrupt_map::PCIInterruptMap;

include!(concat!(env!("OUT_DIR"), "/pcie_server_impl.rs"));

define_server!(PCIEServerStruct {
    buses: Mutex<Vec<pcie::PCIBus>>,
    mem_base: Mutex<usize>,
    io_base: Mutex<usize>,
    interrupt_map: Option<PCIInterruptMap>,
});

define_session!(PCIESession {}, PCIEServerStruct);

impl PCIEServerStruct {
    fn accept_main_session(self: &Arc<PCIEServerStruct>) -> Arc<PCIESession> {
        Arc::new(PCIESession {
            __server: self.clone(),
        })
    }
}

impl PCIESession {
    fn list_devices(&self) -> Vec<PCIDeviceInfo> {
        let mut all_devices = Vec::new();

        let server = self.get_server();
        let buses_locked = server.buses.lock().unwrap();

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

        let server = self.get_server();
        let buses_locked = server.buses.lock().unwrap();
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

        let server = self.get_server();
        let buses_locked = server.buses.lock().unwrap();

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

        let server = self.get_server();
        let mut buses_locked = server.buses.lock().unwrap();

        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        func.inner.header.command = 1 | 2 | 4;
                        return Ok(());
                    }
                }
            }
        }

        Err(OSError::new(Module::Pcie, Reason::NotFound))
    }

    fn get_bar(&self, device: u32, bar_index: u8) -> OSResult<(usize, usize)> {
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let server = self.get_server();
        let mut buses_locked = server.buses.lock().unwrap();

        // just kidding this sucks a lot
        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    // etc
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        // Okay, how big is the BAR?

                        // TODO: Should probably be {read,write}_volatile to ptrs. Probably.
                        // TODO: Handle 64bit BAR properly.

                        let bar_index = bar_index as usize;
                        let old_bar = func.inner.bars[bar_index];

                        let bar_type = old_bar & 0x1;
                        let bar_location = (old_bar & (0x03 << 1)) >> 1;

                        let old_bar_addr: usize = if bar_location == 0b10 {
                            (old_bar as usize & !0xf)
                                | (func.inner.bars[bar_index + 1] as usize) << 32
                        } else {
                            old_bar as usize & !0xf
                        };

                        func.inner.bars[bar_index] = 0xffffffff;
                        let bar_largest = func.inner.bars[bar_index] & !0xf;
                        let bar_size: usize = 0xffffffff - bar_largest as usize + 1;

                        // Ok, put the BAR back? Or make up our own BAR allocation. Augh.
                        let bar_base: usize = if old_bar_addr == 0 {
                            // BARs need to be aligned to their size...
                            let bar_base = if bar_type == 0 {
                                let mut locked = server.mem_base.lock().unwrap();
                                let aligned_up = (*locked + bar_size - 1) & !(bar_size - 1);
                                *locked = aligned_up + bar_size;
                                aligned_up
                            } else {
                                let mut locked = server.io_base.lock().unwrap();
                                let aligned_up = (*locked + bar_size - 1) & !(bar_size - 1);
                                *locked = aligned_up + bar_size;
                                aligned_up
                            };
                            // TODO: "set bar"
                            func.inner.bars[bar_index] = bar_base as u32;
                            bar_base as usize
                        } else {
                            func.inner.bars[bar_index] = (old_bar_addr & 0xffffffff) as u32;
                            old_bar_addr
                        };

                        return Ok((bar_base, bar_size));
                    }
                }
            }
        }

        Err(OSError::new(Module::Pcie, Reason::NotFound))
    }

    fn get_cap(&self, device: u32, cap_index: u8) -> OSResult<Vec<u8>> {
        // just kidding this sucks a lot
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let server = self.get_server();
        let mut buses_locked = server.buses.lock().unwrap();

        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        // TODO: magic number!!!!!!
                        if (func.inner.header.status & (1 << 4)) == (1 << 4) {
                            // caps are supported
                            let cap_offset = func.inner.capabilities;
                            unsafe {
                                // TODO: explicit config space rework
                                let config_space_ptr =
                                    func.inner as *const ecam::ConfigurationSpaceType0 as *const u8;

                                let mut cap_ptr = config_space_ptr.add(cap_offset as usize);
                                let mut curr_cap_index = 0;

                                loop {
                                    let next_cap_offset = *cap_ptr.add(1);

                                    let cap_type = *cap_ptr;
                                    let cap_len = match cap_type {
                                        9 => *cap_ptr.add(2),
                                        0x11 => 0xc,
                                        _ => panic!("Unknown cap type 0x{:x}", cap_type),
                                    };

                                    if cap_index == curr_cap_index {
                                        /* TODO: uhhh. todo?
                                            virtio spec:
                                            For device configuration access, the driver MUST use 8-bit wide accesses for 8-bit wide fields, 16-bit wide
                                            and aligned accesses for 16-bit wide fields and 32-bit wide and aligned accesses for 32-bit and 64-bit wide
                                            fields. For 64-bit fields, the driver MAY access each of the high and low 32-bit parts of the field independently.
                                        */

                                        let cap: Vec<u8> =
                                            std::slice::from_raw_parts(cap_ptr, cap_len as usize)
                                                .to_vec();
                                        return Ok(cap);
                                    }

                                    if next_cap_offset == 0 {
                                        break;
                                    }

                                    curr_cap_index += 1;
                                    cap_ptr = config_space_ptr.add(next_cap_offset as usize);
                                }

                                // TODO: better return code
                                return Err(OSError::new(Module::Pcie, Reason::NotFound));
                            }
                        } else {
                            println!("Caps not supported");
                        }
                    }
                }
            }
        }

        Err(OSError::new(Module::Pcie, Reason::NotFound))
    }

    fn get_interrupt_event(&self, device: u32) -> OSResult<TranslateMoveHandle> {
        // just kidding this sucks a lot
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let server = self.get_server();
        let mut buses_locked = server.buses.lock().unwrap();

        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        let event_handle = syscalls::create_event().unwrap();

                        let interrupt_id = if func.inner.interrupt_line != 0 {
                            func.inner.interrupt_line
                        } else {
                            32 + server
                                .interrupt_map
                                .as_ref()
                                .unwrap()
                                .get_interrupt_id(device_id, func.inner.interrupt_pin)
                                as u8
                        };

                        syscalls::bind_interrupt(event_handle, interrupt_id as usize).unwrap();
                        return Ok(TranslateMoveHandle(event_handle));
                    }
                }
            }
        }
        Err(OSError::new(Module::Pcie, Reason::NotFound))
    }

    fn unbind_interrupt_event(
        &self,
        device: u32,
        event_handle: TranslateCopyHandle,
    ) -> OSResult<()> {
        // just kidding this sucks a lot
        let bus_id: u8 = ((device & (0xff << 16)) >> 16) as u8;
        let device_id: u8 = ((device & (0xff << 8)) >> 8) as u8;
        let function_id: u8 = (device & 0xff) as u8;

        let server = self.get_server();
        let mut buses_locked = server.buses.lock().unwrap();

        for bus in buses_locked.iter_mut() {
            for dev in bus.devices.iter_mut() {
                for func in dev.functions.iter_mut() {
                    if bus.num == bus_id && dev.num == device_id && func.num == function_id {
                        syscalls::unbind_interrupt(
                            event_handle.0,
                            func.inner.interrupt_line as usize,
                        )
                        .unwrap();
                    }
                }
            }
        }
        Err(OSError::new(Module::Pcie, Reason::NotFound))
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
    #[cfg(target_arch = "x86_64")]
    let interrupts = None;

    #[cfg(target_arch = "aarch64")]
    let (pcie_buses, io_space_addr, pci_32bit_addr, _pci_64bit_addr, interrupts) =
        pcie_dt::scan_via_device_tree(0x40000000);

    let port = syscalls::create_port("").unwrap();
    sm::register_port(syscalls::make_tag("pcie"), TranslateCopyHandle(port)).unwrap();

    let server = Arc::new(PCIEServerStruct {
        __server_impl: Mutex::new(ServerImpl::new(port)),
        buses: Mutex::new(pcie_buses),
        mem_base: Mutex::new(pci_32bit_addr.unwrap().try_into().unwrap()),
        io_base: Mutex::new(io_space_addr.unwrap().try_into().unwrap()),
        interrupt_map: interrupts,
    });

    server.process_forever().await;

    syscalls::exit_process();
}
