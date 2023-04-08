use num_enum::{TryFromPrimitive, IntoPrimitive};

#[repr(C)]
#[derive(Debug)]
pub enum MemoryRegionType {
    None,
    Bootloader,
    Memory,
}

#[repr(C)]
#[derive(Debug)]
pub struct MemoryRegion {
    pub start: usize,
    pub length: usize,
    pub ty: MemoryRegionType,
}

// get_system_info values
#[repr(usize)]
#[derive(TryFromPrimitive, IntoPrimitive)]
pub enum SystemInfoType {
    MemoryRegion = 0,
    Platform = 1,
}

#[repr(C)]
#[derive(PartialEq)]
pub enum Platform {
    Virt,
    Pc,
    Raspi3,
    Raspi4
}

#[repr(C)]
pub enum SystemInfo {
    None,
    MemoryRegion(MemoryRegion),
    Platform(Platform)
}
