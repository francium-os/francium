use francium_x86::gdt::*;

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

pub fn setup_gdt() {
    unsafe {
        // setup tss
        let tss_location = &TSS_STORAGE as *const TSS as usize;
        let limit = core::mem::size_of::<TSS>() - 1;

        GDT_ENTRIES[6] = GDTEntry::new_tss_low(tss_location, limit);
        GDT_ENTRIES[7] = GDTEntry::new_tss_high(tss_location, limit);

        francium_x86::gdt::use_gdt(&GDT_ENTRIES);
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
