use core::fmt;
use francium_common::types::FramebufferInfo;
use num_enum::{IntoPrimitive, TryFromPrimitive};

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
    FramebufferInfo = 2,
}

#[repr(C)]
#[derive(PartialEq)]
pub enum Platform {
    Virt,
    Pc,
    Raspi3,
    Raspi4,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Platform::Virt => write!(f, "QEMU virt target"),
            Platform::Pc => write!(f, "PC"),
            Platform::Raspi3 => write!(f, "Raspberry Pi 3"),
            Platform::Raspi4 => write!(f, "Raspberry Pi 4")
        }
    }
}

#[repr(C)]
pub enum SystemInfo {
    None,
    MemoryRegion(MemoryRegion),
    Platform(Platform),
    FramebufferInfo(FramebufferInfo),
}
