pub struct PCIInterruptMap {
    pub map: Vec<[u32; 4]>,
    pub mask: u8
}

impl PCIInterruptMap {
    pub fn get_interrupt_id(&self, device_num: u8, interrupt_line: u8) -> u32 {
        self.map[(device_num & self.mask) as usize][(interrupt_line - 1) as usize]
    }
}
