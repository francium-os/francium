#[repr(C, packed)]
struct GDTR {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
struct GDTEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    flags_limit: u8,
    base_high: u8,
}

#[repr(C, packed)]
pub struct TSS {
    reserved_0: u32,
    pub rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved_1: u64,
    ist: [u64; 7],
    reserved_2: u64,
    reserved_3: u16,
    iomap_base: u16,
}

impl GDTEntry {
    const fn null() -> GDTEntry {
        GDTEntry {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            flags_limit: 0,
            base_high: 0,
        }
    }

    const fn new(base: u32, limit: u32, access: u8, long_mode: bool) -> GDTEntry {
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
}

static mut GDT_ENTRIES: [GDTEntry; 8] = [
    // 0x0
    GDTEntry::null(),

    // 0x08 - Kernel code
    GDTEntry::new(0, 0xfffff, (1 << 7) | (1 << 4) | (1 << 3) | (1 << 1), true),
    // 0x10 - Kernel data
    GDTEntry::new(0, 0xfffff, (1 << 7) | (1 << 4) | (1 << 1), false),
    // 0x18 - User code (32 bit, placeholder)
    GDTEntry::null(),
    // 0x20 - User data
    GDTEntry::new(0, 0xfffff, (1 << 7) | (3 << 5) | (1 << 4) | (1 << 1), false),
    // 0x28 - User code (64 bit)
    GDTEntry::new(
        0,
        0xfffff,
        (1 << 7) | (3 << 5) | (1 << 4) | (1 << 3) | (1 << 1),
        true,
    ),
    // 0x30 - TSS
    GDTEntry::null(),
    GDTEntry::null(),
];

pub static mut TSS_STORAGE: TSS = TSS {
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

extern "C" {
    static interrupt_stack_top: i32;
}

pub fn set_ist() {
    // tec
}

pub fn setup_gdt() {
    unsafe {
        // setup tss
        let tss_location = &TSS_STORAGE as *const TSS as usize;
        let limit = core::mem::size_of::<TSS>() - 1;

        GDT_ENTRIES[6] = GDTEntry {
            limit_low: (limit & 0xffff) as u16,
            base_low: (tss_location & 0xffff) as u16,
            base_middle: ((tss_location & 0xff0000) >> 16) as u8,
            access: 0xe9, //???
            flags_limit: ((limit & 0xf0000) >> 16) as u8 | (0 << 4),
            base_high: ((tss_location & 0xff000000) >> 24) as u8,
        };

        GDT_ENTRIES[7] = GDTEntry {
            limit_low: ((tss_location & 0x0000ffff00000000) >> 32) as u16,
            base_low: ((tss_location & 0xffff000000000000) >> 48) as u16,
            base_middle: 0,
            access: 0,
            flags_limit: 0,
            base_high: 0,
        };

        let gdtr = GDTR {
            limit: (GDT_ENTRIES.len() * 8 - 1) as u16,
            base: &GDT_ENTRIES as *const GDTEntry as u64,
        };

        let gdtr_loc = &gdtr as *const GDTR as usize;

        core::arch::asm!("lgdt [{gdtr_loc}]", gdtr_loc = in (reg) (gdtr_loc));
        core::arch::asm!("mov ax, 0x33; ltr ax");

        TSS_STORAGE.rsp0 = &interrupt_stack_top as *const i32 as u64;
        TSS_STORAGE.rsp1 = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.rsp2 = 0xaaaaaaaaaaaaaaaa;

        TSS_STORAGE.ist[0] = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.ist[1] = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.ist[2] = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.ist[3] = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.ist[4] = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.ist[5] = 0xaaaaaaaaaaaaaaaa;
        TSS_STORAGE.ist[6] = 0xaaaaaaaaaaaaaaaa;
    }
}
