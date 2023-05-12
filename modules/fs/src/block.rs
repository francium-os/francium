pub trait BlockDevice {
    fn read_sector(&mut self, offset: u64, buffer: &mut [u8]) -> u64;
    fn write_sector(&mut self, offset: u64, buffer: &[u8]) -> u64;

    fn get_size(&self) -> u64;
}
