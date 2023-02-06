/* PCIE header */

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct ConfigurationSpaceHeader {
    pub vendor_id: u16,
    pub device_id: u16,

    pub command: u16,
    pub status: u16,

    pub revision: u8,

    pub prog_if: u8,
    pub subclass: u8,
    pub class: u8,

    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct ConfigurationSpaceType0 {
    pub header: ConfigurationSpaceHeader,
    // BARs
    pub bars: [u32; 6],
    _cardbus_cis_pointer: u32, // Reserved, reads 0
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_base: u32,
    pub capabilities: u8,
    _reserved: [u8; 7],
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_gnt: u8,
    pub max_lat: u8,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Type1ConfigurationSpace {/* ConfigurationSpaceHeader */}
