use crate::pcie::PCIBus;

use process::syscalls;
use francium_common::align::align_up;
use francium_common::types::{MapType, PagePermission};
use std::ptr::NonNull;

use acpi::{AcpiHandler, AcpiTables, PciConfigRegions, PhysicalMapping};
use common::constants;

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
        let virt = syscalls::map_device_memory(
            page_addr,
            0,
            new_size,
            MapType::NormalCachable,
            PagePermission::USER_READ_WRITE,
        )
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

// When using ACPI, we assume firmware has already set up BARs etc.
pub fn scan_via_acpi() -> Vec<PCIBus> {
    let acpi_table_base = syscalls::bodge(constants::GET_ACPI_BASE, 0);

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
            MapType::NormalCachable,
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
