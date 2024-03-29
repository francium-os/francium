use core::arch::asm;

#[repr(C, packed)]
struct IDTR {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
pub struct IDTEntry {
    offset_low: u16,
    seg_selector: u16,
    ist: u8,
    type_flags: u8,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}

impl IDTEntry {
    pub const fn null() -> IDTEntry {
        IDTEntry {
            offset_low: 0,
            seg_selector: 0,
            ist: 0,
            type_flags: 0,
            offset_middle: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub const fn new(offset: usize, ist: u8, _flags: u8) -> IDTEntry {
        // cheat a little: hardcode kernel code seg
        IDTEntry {
            offset_low: (offset & 0xffff) as u16,
            seg_selector: 8,
            ist: ist,
            type_flags: (1 << 7) | (0b1110),
            offset_middle: ((offset & 0xffff0000) >> 16) as u16,
            offset_high: ((offset & 0xffffffff00000000) >> 32) as u32,
            reserved: 0,
        }
    }
}

pub fn use_idt(entries: &[IDTEntry]) {
    unsafe {
        let idtr = IDTR {
            limit: (entries.len() * 16 - 1) as u16,
            base: entries.as_ptr() as u64,
        };

        let idtr_loc = &idtr as *const IDTR as usize;

        asm!("lidt [{idtr_loc}]", idtr_loc = in (reg) (idtr_loc));
    }
}
