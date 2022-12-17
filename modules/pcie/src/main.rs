use process::syscalls;
use common::constants;

fn main() {
    println!("Hello from pcie!");

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
