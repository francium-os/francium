#[derive(Debug)]
pub enum MemoryRegionType {
    None,
    Bootloader,
    Memory,
}

#[derive(Debug)]
pub struct MemoryRegion {
    pub start: usize,
    pub length: usize,
    pub ty: MemoryRegionType,
}

// get_system_info values
pub enum SystemInfoType {
    MemoryRegion = 0,
}

pub enum SystemInfo {
    MemoryRegion(MemoryRegion),
}
