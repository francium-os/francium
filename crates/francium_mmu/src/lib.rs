#![no_std]
#![feature(generic_const_exprs)]

use core::marker::PhantomData;
use francium_common::types::*;

// I am going to assume `PageTableEntry` is usize.
// This might bite me later.

type PageTableEntry = usize;
type EntryFlags = usize;

pub trait PageTableSpecific {
    const ENTRIES_PER_LEVEL: usize;

    fn is_valid(e: PageTableEntry) -> bool;
    fn is_table(e: PageTableEntry) -> bool;

    fn map_perms(perm: PagePermission) -> usize;
    fn map_type(ty: MapType) -> usize;
    fn get_page_default_flags() -> EntryFlags;
    fn get_table_default_flags() -> EntryFlags;
    fn get_block_default_flags() -> EntryFlags;

    fn new_entry(flags: EntryFlags, addr: PhysAddr) -> PageTableEntry;
    fn get_addr(e: PageTableEntry) -> PhysAddr;
}

pub trait PhysAlloc {
    fn alloc() -> Option<PhysAddr>;
}

pub trait PhysAccess {
    fn phys_to_virt(p: PhysAddr) -> usize;
}

#[derive(Debug)]
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable<T: PageTableSpecific, A: PhysAlloc, P: PhysAccess> where [(); T::ENTRIES_PER_LEVEL]: {
    entries: [PageTableEntry; T::ENTRIES_PER_LEVEL],
    _t: PhantomData<T>,
    _a: PhantomData<A>,
    _p: PhantomData<P>
}

impl<T: PageTableSpecific, A: PhysAlloc, P: PhysAccess> PageTable<T, A, P> where [(); T::ENTRIES_PER_LEVEL]: {
    pub const fn new() -> PageTable<T, A, P> {
        PageTable {
            entries: [0; T::ENTRIES_PER_LEVEL],
            _t: PhantomData,
            _a: PhantomData,
            _p: PhantomData
        }
    }

    pub fn user_process(&self) -> PageTable<T, A, P> {
        // TODO: is there a better way to do this

        let mut pg = PageTable::<T, A, P>::new();

        pg.entries[448] = self.entries[448];
        pg.entries[480] = self.entries[480];
        pg.entries[511] = self.entries[511];

        pg
    }

    pub fn map_4k(&mut self, phys: PhysAddr, virt: usize, perm: PagePermission, ty: MapType) {
        assert!(phys.is_aligned(0x1000));
        assert!((virt & (0x1000 - 1)) == 0);

        let entry_flags = T::get_page_default_flags() | T::map_perms(perm) | T::map_type(ty);
        let entry = T::new_entry(entry_flags, phys);

        unsafe {
            match self.map_internal(virt, entry, perm, 0, 3) {
                Some(_) => (),
                None => {
                    panic!("4k map failed!");
                }
            }
        }
    }

    pub fn map_2mb(&mut self, phys: PhysAddr, virt: usize, perm: PagePermission, ty: MapType) {
        assert!(phys.is_aligned(0x200000));
        assert!((virt & (0x200000 - 1)) == 0);

        let entry_flags = T::get_block_default_flags() | T::map_perms(perm) | T::map_type(ty);
        let entry = T::new_entry(entry_flags, phys);

        unsafe {
            match self.map_internal(virt, entry, perm, 0, 2) {
                Some(_) => (),
                None => {
                    panic!("2mb map failed!");
                }
            }
        }
    }

    pub fn map_1gb(&mut self, phys: PhysAddr, virt: usize, perm: PagePermission, ty: MapType) {
        assert!(phys.is_aligned(0x40000000));
        assert!((virt & (0x40000000 - 1)) == 0);
        let entry_flags = T::get_block_default_flags() | T::map_perms(perm) | T::map_type(ty);
        let entry = T::new_entry(entry_flags, phys);

        unsafe {
            match self.map_internal(virt, entry, perm, 0, 1) {
                Some(_) => (),
                None => {
                    panic!("1gb map failed!");
                }
            }
        }
    }

    pub fn reprotect_4k(&mut self, virt: usize, perm: PagePermission, ty: MapType) {
        // XXX walk+map is going to be awfully slow
        let addr = self.virt_to_phys(virt).unwrap();
        self.map_4k(addr, virt, perm, ty);
    }

    // XXX TODO: Linux does core::arch::asm!("dsb ishst; isb;"); on aarch64 after modifying PTEs.

    unsafe fn map_internal(
        &mut self,
        virt: usize,
        entry: PageTableEntry,
        perm: PagePermission,
        level: i32,
        final_level: i32,
    ) -> Option<()> {
        let off = (3 - level) * 9 + 12;

        let index = (virt & (0x1ff << off)) >> off;
        if level < final_level {
            let e = self.entries[index];
            if !T::is_valid(e) {
                let new_table_phys: PhysAddr = A::alloc()?;

                let x: usize = P::phys_to_virt(new_table_phys);
                let page_table = x as *mut PageTable<T, A, P>;
                *page_table = PageTable::<T, A, P>::new();

                let new_entry = T::new_entry(T::get_table_default_flags(), new_table_phys);
                self.entries[index] = new_entry;
            }

            let x: usize = P::phys_to_virt(T::get_addr(self.entries[index]));
            let page_table = x as *mut PageTable<T, A, P>;
            page_table
                .as_mut()?
                .map_internal(virt, entry, perm, level + 1, final_level)
        } else {
            // We are the final table! good.
            self.entries[index] = entry;
            Some(())
        }
    }

    unsafe fn walk_internal(&self, virt: usize, level: usize) -> Option<PhysAddr> {
        let final_level = 3;
        let off = (3 - level) * 9 + 12;

        let index = (virt & (0x1ff << off)) >> off;

        let entry = self.entries[index];

        if T::is_valid(entry) {
            if T::is_table(entry) {
                if level < final_level {
                    let x: usize = P::phys_to_virt(T::get_addr(self.entries[index]));
                    let page_table = x as *const PageTable<T, A, P>;
                    page_table.as_ref()?.walk_internal(virt, level + 1)
                } else {
                    // calc block size from level
                    let page_addr = T::get_addr(self.entries[index]).0;
                    let page_mask = (1 << (off)) - 1;
                    Some(PhysAddr((virt & page_mask) + page_addr))
                }
            } else {
                // done, unless we are in the final level
                if level < final_level {
                    let block_addr = T::get_addr(self.entries[index]).0;
                    let block_mask = (1 << (off)) - 1;
                    Some(PhysAddr((virt & block_mask) + block_addr))
                } else {
                    // Block encoding in level 3 table is invalid
                    panic!("Your page tables are broken!");
                }
            }
        } else {
            // not valid, stop
            None
        }
    }

    pub fn virt_to_phys(&self, virt: usize) -> Option<PhysAddr> {
        unsafe { self.walk_internal(virt, 0) }
    }
}