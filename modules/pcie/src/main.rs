use process::syscalls;
use common::constants;
use common::system_info::*;

fn main() {
    println!("Hello from pcie!");

    if let Ok(SystemInfo::MemoryRegion(memory_region)) = syscalls::get_system_info(SystemInfoType::MemoryRegion, 0) {
        println!("memory region 0: {:?}", memory_region);
    }

    let acpi_table_base = syscalls::bodge(constants::GET_ACPI_BASE, 0);
    println!("Acpi table base: {:?}", acpi_table_base);

    let acpi_table_page = acpi_table_base & !0xfff;
    let new_virt = syscalls::map_device_memory(acpi_table_page, 0, 0x1000, 2|4).unwrap();
    println!("mapped: {:x}", new_virt);

    unsafe {
        let new_virt_ptr = (new_virt + (acpi_table_base & 0xfff)) as *const u8;
        println!("{:x}", new_virt_ptr as usize);
        println!("{:?}", core::str::from_utf8(core::slice::from_raw_parts(new_virt_ptr, 8)));
    }

    syscalls::exit_process();
}
