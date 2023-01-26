use crate::ecam::ECAMHeader;
use process::syscalls;
use common::constants;
use common::system_info::*;
use francium_common::align::align_up;
use francium_common::types::{PhysAddr, PagePermission};
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping, PciConfigRegions};
use std::ptr::NonNull;
use smallvec::SmallVec;

use process::ipc_server::ServerImpl;

#[derive(Copy, Clone)]
struct UserACPIHandler {}
impl AcpiHandler for UserACPIHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize
    ) -> PhysicalMapping<Self, T> {
        let page_addr = physical_address & !0xfff;
        let page_offset = physical_address & 0xfff;
        let new_size = align_up(size + page_offset, 0x1000);

        // TODO: make this not, really awful
        let virt = syscalls::map_device_memory(page_addr, 0, new_size, PagePermission::USER_READ_WRITE.bits() as u32).unwrap();
        PhysicalMapping::new(physical_address, NonNull::new((virt + page_offset) as *mut T).unwrap(), size, size, *self)
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        // XXX there's no unmap_device_memory yet. we have enough address space. probably.
    }
}

fn get_function(block: usize, bus: u8, device: u8, function: u8) -> &'static ECAMHeader {
    let offset: usize = ((bus as usize) << 20) + ((device as usize) << 15) + ((function as usize) << 12);
    unsafe {
        ((block + offset) as *const ECAMHeader).as_ref().unwrap()
    }
}

// TODO: hotplug exists

#[derive(Debug)]
pub struct PCIFunction {
    pub num: u8,
    pub inner: &'static ECAMHeader
}

#[derive(Debug)]
pub struct PCIDevice {
    pub num: u8,
    pub functions: SmallVec<[PCIFunction; 1]>
}

impl PCIDevice {
    fn new(block: usize, bus_num: u8, device_num: u8) -> Option<PCIDevice> {
        let mut device_obj = PCIDevice {
            num: device_num,
            functions: SmallVec::new()
        };

        let mut device_ok = false;
        for function_num in 0..=255 {
            let (function_valid, is_multifunction) = device_obj.discover_function(block, bus_num, device_num, function_num);
            if !function_valid {
                break
            }
            device_ok = true;

            if !is_multifunction {
                break
            }
        }

        if device_ok {
            Some(device_obj)
        } else {
            None
        }
    }

    fn discover_function(&mut self, block: usize, bus_num: u8, device_num: u8, function_num: u8) -> (bool /* valid */, bool /* is multifunction */) {
        let function = get_function(block, bus_num, device_num, function_num);
        if function.vendor_id != 0xffff {
            println!("vid: {:04x}, pid: {:04x} type={:04x}", {function.vendor_id}, {function.device_id}, {function.header_type});
            let header_type = function.header_type & 0x7f;
            match header_type {
                /* device */
                0 => {

                }
                /* pci bridge */
                1 => {

                }
                _ => todo!()
            }

            self.functions.push(PCIFunction { num: function_num, inner: function });

            (true, (function.header_type & 0x80)==0x80)
        } else {
            (false, false)
        }
    }
}

#[derive(Debug)]
pub struct PCIBus {
    pub num: u8,
    pub devices: Vec<PCIDevice>
}

impl PCIBus {
    fn new(block: usize, bus_num: u8) -> Option<PCIBus> {
        let mut bus_obj = PCIBus {
            num: bus_num,
            devices: Vec::new()
        };

        let mut bus_ok = false;
        for device_num in 0..=255 {
            match PCIDevice::new(block, bus_num, device_num) {
                Some(some_device) => {
                    bus_obj.devices.push(some_device);
                }

                None => {
                    break
                }
            }

            bus_ok = true;
        }

        if bus_ok {
            Some(bus_obj)
        } else {
            None
        }
    }
}

pub fn scan_via_acpi() -> Vec<PCIBus> {
    let acpi_table_base = syscalls::bodge(constants::GET_ACPI_BASE, 0);
    println!("Acpi table base: {:?}", acpi_table_base);

    let handler = UserACPIHandler{};
    let tables = unsafe { 
        AcpiTables::from_rsdp(handler, acpi_table_base).unwrap()
    };
    
    let pci_regions = PciConfigRegions::new_in(&tables, &std::alloc::System).unwrap();

    let mut buses = Vec::new();

    for ecam_block in pci_regions.iter() {
        //println!("ECAM block: segment_group={}, bus_range={:?}, addr={:08x}", ecam_block.segment_group, ecam_block.bus_range, ecam_block.physical_address);
        let ecam_size: usize = (*ecam_block.bus_range.end() as usize - *ecam_block.bus_range.start() as usize + 1) << 20;
        let ecam_virt = syscalls::map_device_memory(ecam_block.physical_address, 0, ecam_size, PagePermission::USER_READ_WRITE.bits() as u32).unwrap();

        assert!(ecam_block.segment_group == 0);

        for bus_num in ecam_block.bus_range {
            if let Some(pci_bus) = PCIBus::new(ecam_virt, bus_num) {
                //println!("bus: {:?}", pci_bus);

                buses.push(pci_bus);
            }
        }
    }

    buses
}