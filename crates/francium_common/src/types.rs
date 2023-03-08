#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct PhysAddr(pub usize);

impl core::fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl PhysAddr {
    pub fn is_aligned(&self, n: usize) -> bool {
        self.0 & (n - 1) == 0
    }
}

bitflags! {
    pub struct PagePermission : u64 {
        const READ_ONLY = 0;
        const WRITE = 1;
        const EXECUTE = 2;
        const KERNEL = 4;

        const USER_READ_ONLY = Self::READ_ONLY.bits;
        const USER_READ_WRITE = Self::READ_ONLY.bits | Self::WRITE.bits;
        const USER_READ_EXECUTE = Self::READ_ONLY.bits | Self::EXECUTE.bits;
        const USER_RWX = Self::READ_ONLY.bits | Self::WRITE.bits | Self::EXECUTE.bits;

        const KERNEL_READ_ONLY = Self::READ_ONLY.bits | Self::KERNEL.bits;
        const KERNEL_READ_WRITE = Self::READ_ONLY.bits | Self::WRITE.bits | Self::KERNEL.bits;
        const KERNEL_READ_EXECUTE = Self::READ_ONLY.bits| Self::EXECUTE.bits | Self::KERNEL.bits;
        const KERNEL_RWX = Self::KERNEL_READ_EXECUTE.bits | Self::WRITE.bits;
    }
}

use num_derive::FromPrimitive;
#[derive(Copy, Clone, FromPrimitive)]
pub enum MapType {
    NormalCachable,
    NormalUncachable,
    Device,
}
