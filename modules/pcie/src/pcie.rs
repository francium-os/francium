use crate::ecam::*;
use common::constants;
use process::syscalls;
//use common::system_info::*;
use acpi::{AcpiHandler, AcpiTables, PciConfigRegions, PhysicalMapping};
use francium_common::align::align_up;
use francium_common::types::PagePermission;
use smallvec::SmallVec;
use std::ptr::NonNull;

use fdt_rs::base::*;
use fdt_rs::index::*;
use fdt_rs::prelude::*;

#[derive(Copy, Clone)]
struct UserACPIHandler {}
impl AcpiHandler for UserACPIHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let page_addr = physical_address & !0xfff;
        let page_offset = physical_address & 0xfff;
        let new_size = align_up(size + page_offset, 0x1000);

        // TODO: make this not, really awful
        let virt =
            syscalls::map_device_memory(page_addr, 0, new_size, PagePermission::USER_READ_WRITE)
                .unwrap();
        PhysicalMapping::new(
            physical_address,
            NonNull::new((virt + page_offset) as *mut T).unwrap(),
            size,
            size,
            *self,
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // XXX there's no unmap_device_memory yet. we have enough address space. probably.
    }
}

fn get_function_header(
    block: usize,
    bus: u8,
    device: u8,
    function: u8,
) -> &'static mut ConfigurationSpaceHeader {
    let offset: usize =
        ((bus as usize) << 20) + ((device as usize) << 15) + ((function as usize) << 12);
    unsafe {
        ((block + offset) as *mut ConfigurationSpaceHeader)
            .as_mut()
            .unwrap()
    }
}

fn get_function_type0(
    block: usize,
    bus: u8,
    device: u8,
    function: u8,
) -> &'static mut ConfigurationSpaceType0 {
    let offset: usize =
        ((bus as usize) << 20) + ((device as usize) << 15) + ((function as usize) << 12);
    unsafe {
        ((block + offset) as *mut ConfigurationSpaceType0)
            .as_mut()
            .unwrap()
    }
}

// TODO: hotplug exists

#[derive(Debug)]
pub struct PCIFunction {
    pub num: u8,
    pub inner: &'static mut ConfigurationSpaceType0,
}

#[derive(Debug)]
pub struct PCIDevice {
    pub num: u8,
    pub functions: SmallVec<[PCIFunction; 1]>,
}

impl PCIDevice {
    fn new(block: usize, bus_num: u8, device_num: u8) -> Option<PCIDevice> {
        let mut device_obj = PCIDevice {
            num: device_num,
            functions: SmallVec::new(),
        };

        let mut device_ok = false;
        for function_num in 0..=255 {
            let (function_valid, is_multifunction) =
                device_obj.discover_function(block, bus_num, device_num, function_num);
            if !function_valid {
                break;
            }
            device_ok = true;

            if !is_multifunction {
                break;
            }
        }

        if device_ok {
            Some(device_obj)
        } else {
            None
        }
    }

    fn discover_function(
        &mut self,
        block: usize,
        bus_num: u8,
        device_num: u8,
        function_num: u8,
    ) -> (bool /* valid */, bool /* is multifunction */) {
        let function = get_function_header(block, bus_num, device_num, function_num);

        if function.vendor_id != 0xffff {
            println!(
                "vid: {:04x}, pid: {:04x} type={:04x}",
                { function.vendor_id },
                { function.device_id },
                { function.header_type }
            );
            let header_type = function.header_type & 0x7f;
            match header_type {
                /* device */
                0 => {
                    let function_type0 =
                        get_function_type0(block, bus_num, device_num, function_num);
                    for bar in function_type0.bars {
                        println!(
                            "{:08x}: Bar type: {}, location: 0b{:02b}, prefetchable: {}",
                            bar,
                            bar & 1,
                            (bar & (3 << 1)) >> 1,
                            (bar & (1 << 3)) >> 3
                        );
                    }
                    self.functions.push(PCIFunction {
                        num: function_num,
                        inner: function_type0,
                    });
                }
                /* pci bridge */
                1 => {}
                _ => todo!(),
            }

            (true, (function.header_type & 0x80) == 0x80)
        } else {
            (false, false)
        }
    }
}

#[derive(Debug)]
pub struct PCIBus {
    pub num: u8,
    pub devices: Vec<PCIDevice>,
}

impl PCIBus {
    fn new(block: usize, bus_num: u8) -> Option<PCIBus> {
        let mut bus_obj = PCIBus {
            num: bus_num,
            devices: Vec::new(),
        };

        let mut bus_ok = false;
        for device_num in 0..=255 {
            match PCIDevice::new(block, bus_num, device_num) {
                Some(some_device) => {
                    bus_obj.devices.push(some_device);
                }

                None => break,
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

    let handler = UserACPIHandler {};
    let tables = unsafe { AcpiTables::from_rsdp(handler, acpi_table_base).unwrap() };

    let pci_regions = PciConfigRegions::new_in(&tables, &std::alloc::System).unwrap();

    let mut buses = Vec::new();

    for ecam_block in pci_regions.iter() {
        //println!("ECAM block: segment_group={}, bus_range={:?}, addr={:08x}", ecam_block.segment_group, ecam_block.bus_range, ecam_block.physical_address);
        let ecam_size: usize =
            (*ecam_block.bus_range.end() as usize - *ecam_block.bus_range.start() as usize + 1)
                << 20;
        let ecam_virt = syscalls::map_device_memory(
            ecam_block.physical_address,
            0,
            ecam_size,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap();

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

pub fn scan_via_device_tree(dt_addr: usize) -> Vec<PCIBus> {
    // Does this suck? yes it does lmao

    let mut buses = Vec::new();

    let dt_header_virt =
        syscalls::map_device_memory(dt_addr, 0, 0x1000, PagePermission::USER_READ_WRITE).unwrap();
    let dt_len = unsafe {
        DevTree::read_totalsize(std::slice::from_raw_parts(
            dt_header_virt as *const u8,
            0x1000,
        ))
    }
    .unwrap();

    let dt_virt = syscalls::map_device_memory(
        dt_addr,
        0,
        align_up(dt_len as usize, 0x1000),
        PagePermission::USER_READ_WRITE,
    )
    .unwrap();
    let fdt = unsafe { DevTree::from_raw_pointer(dt_virt as *const u8).unwrap() };

    let layout = DevTreeIndex::get_layout(&fdt).unwrap();
    let mut vec = vec![0u8; layout.size() + layout.align()];
    let raw_slice = vec.as_mut_slice();
    let index = DevTreeIndex::new(fdt, raw_slice).unwrap();

    let root_node = index.root();
    let mut root_props = root_node.props();
    while let Some(prop) = root_props.next() {
        let name = prop.name().unwrap();
        if name == "#size-cells" {
            assert!(prop.u32(0).unwrap() == 2);
        } else if name == "#address-cells" {
            assert!(prop.u32(0).unwrap() == 2);
        }
    }

    let mut pcie_iter = index.compatible_nodes("pci-host-ecam-generic");
    while let Some(node) = pcie_iter.next() {
        // Make sure the address size is correct ...
        let mut prop_iter = node.props();
        while let Some(prop) = prop_iter.next() {
            let name = prop.name().unwrap();
            // assert that address-cells=3, size-cells=2
            if name == "#size-cells" {
                assert!(prop.u32(0).unwrap() == 2);
            } else if name == "#address-cells" {
                assert!(prop.u32(0).unwrap() == 3);
            }
        }

        let mut prop_iter = node.props();
        while let Some(prop) = prop_iter.next() {
            let name = prop.name().unwrap();

            if name == "ranges" {
                println!("Pain!");
                let num_ranges = prop.length() / (4 * 7);
                for i in (0..num_ranges * 7).step_by(7) {
                    let pci_hi = prop.u32(i).unwrap();
                    let pci_addr =
                        prop.u32(i + 2).unwrap() as u64 | (prop.u32(i + 1).unwrap() as u64) << 32;
                    let host_addr =
                        prop.u32(i + 4).unwrap() as u64 | (prop.u32(i + 3).unwrap() as u64) << 32;
                    let host_size =
                        prop.u32(i + 6).unwrap() as u64 | (prop.u32(i + 5).unwrap() as u64) << 32;
                    println!(
                        "pci: ({:08x} {:x}) host ({:x}, {:x})",
                        pci_hi, pci_addr, host_addr, host_size
                    );
                }
            } else if name == "reg" {
                let ecam_addr = prop.u32(1).unwrap() as u64 | (prop.u32(0).unwrap() as u64) << 32;
                let ecam_size = prop.u32(3).unwrap() as u64 | (prop.u32(2).unwrap() as u64) << 32;
                println!("ecam: {:x} {:x}", ecam_addr, ecam_size);

                let ecam_virt = syscalls::map_device_memory(
                    ecam_addr as usize,
                    0,
                    ecam_size as usize,
                    PagePermission::USER_READ_WRITE,
                )
                .unwrap();

                // finally, buses!
                for bus_num in 0..=255 {
                    if let Some(pci_bus) = PCIBus::new(ecam_virt, bus_num as u8) {
                        buses.push(pci_bus);
                    }
                }
            }
        }
    }

    buses
}
