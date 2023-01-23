use process::syscalls;
use common::constants;
use common::system_info::*;
use francium_common::align::align_up;
use francium_common::types::{PhysAddr, PagePermission};

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping, PciConfigRegions};

use std::ptr::NonNull;

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
        // TODO! there's no unmap_device_memory yet. we have enough address space _stares_
        println!("Unmap!");
    }
}

fn main() {
    println!("Hello from pcie!");

    let acpi_table_base = syscalls::bodge(constants::GET_ACPI_BASE, 0);
    println!("Acpi table base: {:?}", acpi_table_base);

    let handler = UserACPIHandler{};
    let tables = unsafe { 
        AcpiTables::from_rsdp(handler, acpi_table_base).unwrap()
    };
    
    let pci_regions = PciConfigRegions::new_in(&tables, &std::alloc::System).unwrap();
    let root_bridge_region = pci_regions.physical_address(0, 0, 0, 0).unwrap();
    println!("PCI region: {:x}", root_bridge_region);

    syscalls::exit_process();
}
