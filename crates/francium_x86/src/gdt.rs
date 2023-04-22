#[repr(C, packed)]
struct GDTR {
    limit: u16,
    base: u64,
}

#[derive(Default)]
#[repr(C, packed)]
pub struct GDTEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    flags_limit: u8,
    base_high: u8,
}

#[derive(Default)]
#[repr(C, packed)]
pub struct TSS {
    pub reserved_0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved_1: u64,
    pub ist: [u64; 7],
    pub reserved_2: u64,
    pub reserved_3: u16,
    pub iomap_base: u16
}

impl TSS {
    pub const DEFAULT: TSS = TSS {
        reserved_0: 0,
        rsp0: 0,
        rsp1: 0,
        rsp2: 0,
        reserved_1: 0,
        ist: [0; 7],
        reserved_2: 0,
        reserved_3: 0,
        iomap_base: 104,
    };
}

impl GDTEntry {
    pub const DEFAULT: GDTEntry = GDTEntry {
        limit_low: 0,
        base_low: 0,
        base_middle: 0,
        access: 0,
        flags_limit: 0,
        base_high: 0,
    };

    pub const fn null() -> GDTEntry {
        GDTEntry::DEFAULT
    }

    pub const fn new(base: u32, limit: u32, access: u8, long_mode: bool) -> GDTEntry {
        // gran: 1, db: 1
        let mut flags: u8 = 1 << 3;
        if long_mode {
            flags |= 1 << 1;
        } else {
            flags |= 1 << 2;
        }
        GDTEntry {
            limit_low: (limit & 0xffff) as u16,
            base_low: (base & 0xffff) as u16,
            base_middle: ((base & 0xff0000) >> 16) as u8,
            access: access,
            flags_limit: ((limit & 0xf0000) >> 16) as u8 | (flags << 4),
            base_high: ((base & 0xff000000) >> 24) as u8,
        }
    }

    pub const fn new_tss_low(tss_location: usize, limit: usize) -> GDTEntry {
        GDTEntry {
            limit_low: (limit & 0xffff) as u16,
            base_low: (tss_location & 0xffff) as u16,
            base_middle: ((tss_location & 0xff0000) >> 16) as u8,
            access: 0xe9, //???
            flags_limit: ((limit & 0xf0000) >> 16) as u8 | (0 << 4),
            base_high: ((tss_location & 0xff000000) >> 24) as u8,
        }
    }

    pub const fn new_tss_high(tss_location: usize, _limit: usize) -> GDTEntry {
        GDTEntry {
            limit_low: ((tss_location & 0x0000ffff00000000) >> 32) as u16,
            base_low: ((tss_location & 0xffff000000000000) >> 48) as u16,
            base_middle: 0,
            access: 0,
            flags_limit: 0,
            base_high: 0,
        }
    }
}

pub fn use_gdt(entries: &[GDTEntry]) {
    unsafe {
        let gdtr = GDTR {
            limit: (entries.len() * 8 - 1) as u16,
            base: entries.as_ptr() as u64,
        };

        let gdtr_loc = &gdtr as *const GDTR as usize;

        core::arch::asm!("lgdt [{gdtr_loc}]", gdtr_loc = in (reg) (gdtr_loc));
    }
}
