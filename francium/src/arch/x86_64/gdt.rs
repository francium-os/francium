use francium_x86::gdt::*;
use crate::per_cpu;

const GDT_ENTRIES: [GDTEntry; 8] = [
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

extern "C" {
    static interrupt_stack_top: i32;
}

pub fn setup_gdt() {
    unsafe {
        // setup tss

        per_cpu::get().gdt = GDT_ENTRIES;
        let gdt = &mut per_cpu::get().gdt;

        let tss = &mut per_cpu::get().tss;
        let tss_location = tss as *const TSS as usize;
        let limit = core::mem::size_of::<TSS>() - 1;

        gdt[6] = GDTEntry::new_tss_low(tss_location, limit);
        gdt[7] = GDTEntry::new_tss_high(tss_location, limit);

        francium_x86::gdt::use_gdt(gdt);
        core::arch::asm!("mov ax, 0x33; ltr ax");

        tss.rsp0 = &interrupt_stack_top as *const i32 as u64;
        tss.rsp1 = 0xaaaaaaaaaaaaaaaa;
        tss.rsp2 = 0xaaaaaaaaaaaaaaaa;

        tss.ist[0] = 0xaaaaaaaaaaaaaaaa;
        tss.ist[1] = 0xaaaaaaaaaaaaaaaa;
        tss.ist[2] = 0xaaaaaaaaaaaaaaaa;
        tss.ist[3] = 0xaaaaaaaaaaaaaaaa;
        tss.ist[4] = 0xaaaaaaaaaaaaaaaa;
        tss.ist[5] = 0xaaaaaaaaaaaaaaaa;
        tss.ist[6] = 0xaaaaaaaaaaaaaaaa;
    }
}
