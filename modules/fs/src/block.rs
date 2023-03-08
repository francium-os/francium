pub trait BlockDevice {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> usize;
    fn write(&self, offset: usize, buffer: &[u8]) -> usize;
}
