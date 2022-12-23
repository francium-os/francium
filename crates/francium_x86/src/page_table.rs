use francium_common::types::*;
use francium_mmu;

bitflags! {
    struct EntryFlags: usize {
        const VALID = 1 << 0;
        const WRITABLE = 1<<1;
        const USER = 1<<2;

        const WRITE_THROUGH = 1<<3;
        const UNCACHEABLE = 1<<4;
        const ACCESS = 1 << 5;

        const DIRTY = 1<<6;

        // Set this to map a (2mb, 1gb) block, leave unset for tables
        // Intel calls it PAGE_SIZE?
        const TYPE_BLOCK = 1 << 7;
        const TYPE_TABLE = 0 << 7;

        // Except for pages, where..
        const PAT = 1<<7;

        // global?? something something kernel
        const GLOBAL = 1<<8;

        const XN = 1<<63;
    }
}

pub struct X86_64Specific {}
impl francium_mmu::PageTableSpecific for X86_64Specific {
    const ENTRIES_PER_LEVEL: usize = 512;

    fn is_valid(entry: usize) -> bool {
        (entry & EntryFlags::VALID.bits) != 0
    }

    fn is_table(entry: usize) -> bool {
        (entry & EntryFlags::TYPE_TABLE.bits) == EntryFlags::TYPE_TABLE.bits
    }

    fn map_perms(perm: PagePermission) -> usize {
        let mut flags: EntryFlags = EntryFlags::empty();

        if !perm.contains(PagePermission::KERNEL) {
            flags |= EntryFlags::USER;
        }

        if perm.contains(PagePermission::WRITE) {
            flags |= EntryFlags::WRITABLE;
        }

        if !perm.contains(PagePermission::EXECUTE) {
            flags |= EntryFlags::XN;
        }

        flags.bits
    }

    fn map_type(ty: MapType) -> usize {
        match ty {
            MapType::NormalCachable => EntryFlags::empty(),
            MapType::NormalUncachable => EntryFlags::UNCACHEABLE,
            MapType::Device => EntryFlags::UNCACHEABLE, // ???
        }.bits
    }

    fn get_page_default_flags() -> usize {
        (EntryFlags::VALID | EntryFlags::ACCESS).bits
    }

    fn get_table_default_flags() -> usize {
        (EntryFlags::VALID | EntryFlags::TYPE_TABLE | EntryFlags::WRITABLE | EntryFlags::USER).bits
    }

    fn get_block_default_flags() -> usize {
        (EntryFlags::VALID | EntryFlags::TYPE_BLOCK | EntryFlags::ACCESS).bits
    }

    fn new_entry(flags: usize, addr: PhysAddr) -> usize {
        (flags & !0x000f_ffff_ffff_f000) | (addr.0 & 0x000f_ffff_ffff_f000)
    }

    fn get_addr(entry: usize) -> PhysAddr {
        PhysAddr(entry & 0x000f_ffff_ffff_f000)
    }
}