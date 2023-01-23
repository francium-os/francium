#![no_std]

#[macro_use]
extern crate log;

use francium_common::types::PhysAddr;
use francium_mmu::PhysAccess;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct RSDP32Bit {
    pub magic: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct RSDP64Bit {
    pub magic: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,

    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub padding: [u8; 3]
}

pub enum RSDP {
    Normal(RSDP32Bit),
    Extended(RSDP64Bit)
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct TableHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32
}

const HEADER_SIZE: usize = core::mem::size_of::<TableHeader>();

pub fn parse_rsdp<P: PhysAccess>(phys: PhysAddr) -> RSDP {
    let virt = P::phys_to_virt(phys);
    unsafe {
        let rdsp_32_bit: *const RSDP32Bit = virt as *const RSDP32Bit; 

        debug!("{:?}", core::str::from_utf8(&(*rdsp_32_bit).magic));
        debug!("acpi revision: {}", (*rdsp_32_bit).revision);
        match (*rdsp_32_bit).revision {
            // TODO: Checksum
            0 => RSDP::Normal(*rdsp_32_bit),
            _ => {
                unimplemented!();
            }
        }
    }
}

pub fn parse_table<P: PhysAccess>(phys: PhysAddr) -> TableHeader {
    let virt = P::phys_to_virt(phys);
    unsafe {
        let header: *const TableHeader = virt as *const TableHeader;

        debug!("Got table: {:?}", core::str::from_utf8(&(*header).signature));
        // TODO: Checksum?

        *header
    }
}

pub fn parse_rsdt<P: PhysAccess>(phys: PhysAddr) -> TableHeader {
    let header = parse_table::<P>(phys);
    assert!(header.signature == *b"RSDT");

    let virt = P::phys_to_virt(phys);
    unsafe {
        let rsdt_length = header.length as usize - HEADER_SIZE;
        for i in (HEADER_SIZE..HEADER_SIZE+rsdt_length).step_by(4) {
            let table_location = *((virt+i) as *const u32);
            let inner_header = parse_table::<P>(PhysAddr(table_location as usize));

            let _virt = P::phys_to_virt(PhysAddr(table_location as usize));

            match &inner_header.signature {
                /*b"MCFG" => {
                    parse_mcfg(virt + HEADER_SIZE, inner_header.length as usize - HEADER_SIZE);
                }*/
                _ => {
                    debug!("Unhandled table {:?}!", inner_header.signature);
                }
            }

            //let inner_len = inner_header.length;
            //println!("{:x}", inner_len as usize - HEADER_SIZE - 8);
        }
        header
    }
}
