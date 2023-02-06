use francium_common::types::*;
use francium_mmu;

bitflags! {
    pub struct EntryFlags: usize {
        // Descriptor bit[0] identifies whether the descriptor is valid, and is 1 for a valid descriptor. I
        const VALID = 1 << 0;
        // Descriptor bit[1] identifies the descriptor type, and is encoded as:
        // 0, Block
        // 1, Table

        const TYPE_BLOCK = 0 << 1;
        const TYPE_TABLE = 1 << 1;

        const TYPE_PAGE = 1 << 1;

        const ATTR_INDEX_0 = 0 << 2;
        const ATTR_INDEX_1 = 1 << 2;
        const ATTR_INDEX_2 = 2 << 2;
        const ATTR_INDEX_3 = 3 << 2;
        const ATTR_INDEX_4 = 4 << 2;
        const ATTR_INDEX_5 = 5 << 2;
        const ATTR_INDEX_6 = 6 << 2;
        const ATTR_INDEX_7 = 7 << 2;

        const ATTR_AP_2 = 1 << 7;
        const ATTR_AP_1 = 1 << 6;

        const ATTR_ACCESS = 1 << 10;

        const ATTR_XN = 1<<54;
        const ATTR_PXN = 1<<53;

        // TODO: uhh, upper half attributes ig
        // • In Armv8.0, the position and contents of bits[63:52, 11:2] are identical to bits[63:52, 11:2] in the Page descriptors.
    }

    // For blocks at level 0:
    // 512GB (Not supported without DS)
    // For blocks at level 1:
    // 1GB
    // For blocks at level 2:
    // 2MB
    // Blocks at level 3 are illegal
}

pub struct AArch64Specific {}
impl francium_mmu::PageTableSpecific for AArch64Specific {
    const ENTRIES_PER_LEVEL: usize = 512;

    fn is_valid(entry: usize) -> bool {
        (entry & EntryFlags::VALID.bits) != 0
    }

    fn is_table(entry: usize) -> bool {
        (entry & EntryFlags::TYPE_TABLE.bits) == EntryFlags::TYPE_TABLE.bits
    }

    fn map_perms(perm: PagePermission) -> usize {
        let mut flags: EntryFlags = EntryFlags::empty();

        // TODO: PXN, maybe

        if !perm.contains(PagePermission::KERNEL) {
            flags |= EntryFlags::ATTR_AP_1;
        }

        if !perm.contains(PagePermission::WRITE) {
            flags |= EntryFlags::ATTR_AP_2;
        }

        if !perm.contains(PagePermission::EXECUTE) {
            flags |= EntryFlags::ATTR_XN;
        }

        flags.bits
    }

    fn map_type(ty: MapType) -> usize {
        match ty {
            MapType::NormalCachable => EntryFlags::ATTR_INDEX_0,
            MapType::NormalUncachable => EntryFlags::ATTR_INDEX_1,
            MapType::Device => EntryFlags::ATTR_INDEX_2,
        }
        .bits
    }

    fn get_page_default_flags() -> usize {
        (EntryFlags::VALID | EntryFlags::TYPE_PAGE | EntryFlags::ATTR_ACCESS).bits
    }

    fn get_table_default_flags() -> usize {
        (EntryFlags::VALID | EntryFlags::TYPE_TABLE).bits
    }

    fn get_block_default_flags() -> usize {
        (EntryFlags::VALID | EntryFlags::TYPE_BLOCK | EntryFlags::ATTR_ACCESS).bits
    }

    fn new_entry(flags: usize, addr: PhysAddr) -> usize {
        (flags & !0x000f_ffff_ffff_f000) | (addr.0 & 0x000f_ffff_ffff_f000)
    }

    fn get_addr(entry: usize) -> PhysAddr {
        PhysAddr(entry & 0x000f_ffff_ffff_f000)
    }
}
