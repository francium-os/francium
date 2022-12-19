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
