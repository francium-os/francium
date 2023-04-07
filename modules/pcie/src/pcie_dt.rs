use crate::pcie::PCIBus;

use common::MapType;
use francium_common::align::align_up;
use francium_common::types::PagePermission;
use process::syscalls;

use fdt_rs::base::*;
use fdt_rs::index::*;
use fdt_rs::prelude::*;

use crate::interrupt_map::PCIInterruptMap;

fn interrupt_map_from_dt(prop: DevTreeIndexProp, mask_value: u8) -> PCIInterruptMap {
    let mut map = PCIInterruptMap {
        map: Vec::new(),
        mask: mask_value
    };

    assert!(prop.length() % 40 == 0);
    /* Format: address-cells, interrupt-cells, interrupt controller phandle, interrupt specifier */

    for i in 0..prop.length() / 160 {
        let mut interrupts: [u32; 4] = [0; 4];

        for j in 0..4 {
            let off = i*40 + j*10;

            let pci_addr_specifier = [prop.u32(off).unwrap(), prop.u32(off + 1).unwrap(), prop.u32(off + 2).unwrap()];
            let pci_interrupt_id = prop.u32(off + 3).unwrap();
            //let phandle = prop.u32(i * 10 + 4).unwrap();
            // GIC interrupt
            let gic_id = prop.u32(off + 8).unwrap();
            // level

            let pci_device = pci_addr_specifier[0] >> 11;
            assert!(pci_interrupt_id == (j+1) as u32);
            assert!(pci_device == i as u32);
            interrupts[j] = gic_id;
        }
        map.map.push(interrupts)
    }
    map
}

// When using Device Tree, we assume firmware has _not_ setup BARs etc.
pub fn scan_via_device_tree(
    dt_addr: usize,
) -> (Vec<PCIBus>, Option<usize>, Option<usize>, Option<usize>, Option<PCIInterruptMap>) {
    // Does this suck? yes it does lmao

    let mut io_space_addr: Option<usize> = None;
    let mut pci_32bit_addr: Option<usize> = None;
    let mut pci_64bit_addr: Option<usize> = None;
    let mut interrupt_map: Option<PCIInterruptMap> = None;

    /* internal only */
    let mut interrupt_map_mask: Option<u8> = None;

    let mut buses = Vec::new();

    let dt_header_virt = syscalls::map_device_memory(
        dt_addr,
        0,
        0x1000,
        MapType::NormalCachable,
        PagePermission::USER_READ_WRITE,
    )
    .unwrap();
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
        MapType::NormalCachable,
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
                let num_ranges = prop.length() / (4 * 7);
                for i in (0..num_ranges * 7).step_by(7) {
                    let pci_hi = prop.u32(i).unwrap();
                    let pci_addr =
                        prop.u32(i + 2).unwrap() as u64 | (prop.u32(i + 1).unwrap() as u64) << 32;
                    let host_addr =
                        prop.u32(i + 4).unwrap() as u64 | (prop.u32(i + 3).unwrap() as u64) << 32;
                    let host_size =
                        prop.u32(i + 6).unwrap() as u64 | (prop.u32(i + 5).unwrap() as u64) << 32;

                    /* from https://elinux.org/Device_Tree_Usage#PCI_Address_Translation:
                       phys.hi cell: npt000ss bbbbbbbb dddddfff rrrrrrrr

                        n: relocatable region flag (doesn't play a role here)
                        p: prefetchable (cacheable) region flag
                        t: aliased address flag (doesn't play a role here)
                        ss: space code
                        00: configuration space
                        01: I/O space
                        10: 32 bit memory space
                        11: 64 bit memory space
                        bbbbbbbb: The PCI bus number. PCI may be structured hierarchically. So we may have PCI/PCI bridges which will define sub busses.
                        ddddd: The device number, typically associated with IDSEL signal connections.
                        fff: The function number. Used for multifunction PCI devices.
                        rrrrrrrr: Register number; used for configuration cycles.
                    */
                    // Also see https://www.openfirmware.info/data/docs/bus.pci.pdf

                    let _is_prefetchable = (pci_hi & 1 << 30) != 0;
                    let pci_space_type = (pci_hi & 3 << 24) >> 24;

                    match pci_space_type {
                        0 /* Configuration space */ => {},
                        1 /* I/O space */ => {
                            io_space_addr.replace(host_addr as usize);
                        },
                        2 /* 32-bit memory */ => {
                            assert!(pci_addr == host_addr);
                            pci_32bit_addr.replace(host_addr as usize);
                        },
                        3 /* 64-bit memory */ => {
                            assert!(pci_addr == host_addr);
                            pci_64bit_addr.replace(host_addr as usize);
                        }
                        _ => {}
                    }

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
                    // TODO: Should ECAM be Device?
                    MapType::NormalCachable,
                    PagePermission::USER_READ_WRITE,
                )
                .unwrap();

                // finally, buses!
                for bus_num in 0..=255 {
                    if let Some(pci_bus) = PCIBus::new(ecam_virt, bus_num as u8) {
                        buses.push(pci_bus);
                    }
                }
            } else if name == "interrupt-map" {
                if let Some(x) = interrupt_map_mask {
                    interrupt_map = Some(interrupt_map_from_dt(prop, x));
                }
            } else if name == "interrupt-map-mask" {
                interrupt_map_mask = Some(((prop.u32(0).unwrap() >> 11) & 0x1f) as u8);
            }
        }
    }

    (buses, io_space_addr, pci_32bit_addr, pci_64bit_addr, interrupt_map)
}
