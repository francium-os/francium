use crate::mmu::{MapType, PagePermission, PageTable};
use crate::phys_allocator;
use francium_common::types::PhysAddr;
use smallvec::SmallVec;
use spin::RwLock;

use crate::arch;

lazy_static! {
    pub static ref KERNEL_ADDRESS_SPACE: RwLock<AddressSpace> =
        RwLock::new(AddressSpace::new(PageTable::new()));
}

#[derive(Debug)]
pub struct Block {
    pub address: usize,
    pub size: usize,
    pub permissions: PagePermission,
}

pub struct AddressSpace {
    pub page_table: &'static mut PageTable,
    page_table_phys: PhysAddr,
    pub regions: SmallVec<[Block; 4]>,
}

impl core::fmt::Debug for AddressSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AddressSpace").finish()
    }
}

fn map_region(pg: &mut PageTable, start_addr: usize, size: usize, perm: PagePermission) {
    unsafe {
        for addr in (start_addr..(start_addr + size)).step_by(0x1000) {
            let page = phys_allocator::alloc().unwrap();
            pg.map_4k(page, addr, perm, MapType::NormalCachable);
        }
    }
}

fn reprotect_region(pg: &mut PageTable, start_addr: usize, size: usize, perm: PagePermission) {
    for addr in (start_addr..(start_addr + size)).step_by(0x1000) {
        pg.reprotect_4k(addr, perm, MapType::NormalCachable);
    }
}

impl AddressSpace {
    pub fn new(template_page_table: PageTable) -> AddressSpace {
        // Crimes activated
        // This will only really work if pagetable is exactly a page big... and we never free it.
        unsafe {
            let phys_page = match phys_allocator::alloc() {
                Some(x) => x,
                None => panic!("Out of physical memory!"),
            };

            let page_table_ptr = crate::mmu::phys_to_virt(phys_page) as *mut PageTable;
            page_table_ptr.copy_from(&template_page_table as *const PageTable, 1);

            let page_table = match page_table_ptr.as_mut() {
                Some(x) => x,
                None => panic!("Somehow phys_to_virt returned null?"),
            };

            AddressSpace {
                page_table: page_table,
                page_table_phys: phys_page,
                regions: SmallVec::new(),
            }
        }
    }

    pub fn alias(
        &mut self,
        start_phys: PhysAddr,
        start_addr: usize,
        size: usize,
        map_type: MapType,
        perm: PagePermission,
    ) {
        for addr in (start_addr..(start_addr + size)).step_by(0x1000) {
            let page = PhysAddr(start_phys.0 + (addr - start_addr));
            self.page_table
                .map_4k(page, addr, perm, map_type);
        }

        self.regions.push(Block {
            address: start_addr,
            size: size,
            permissions: perm,
        })
    }

    pub fn create_with_overlap(&mut self, start_addr: usize, size: usize, perm: PagePermission) {
        let mut found_overlap = false;
        for reg in self.regions.iter_mut() {
            // NOTE: > not >=, if == we do not need to move start
            if reg.address > start_addr && reg.address <= start_addr + size {
                // This region's start is inside the new region.
                //let deficit = reg.address - start_addr;
                panic!("panik");
                //found_overlap = true;
            }

            // NOTE: again, > not >=
            if reg.address + reg.size > start_addr && reg.address + reg.size <= start_addr + size {
                // This region's end is inside the new region.
                let overlap = reg.address + reg.size - start_addr;
                let deficit = size - overlap;
                if reg.permissions != perm {
                    reprotect_region(&mut self.page_table, start_addr, overlap, perm);
                }

                // Need to map a chunk from found region end to new region end.
                map_region(&mut self.page_table, start_addr + overlap, deficit, perm);
                reg.size = size;

                found_overlap = true;
            }

            if found_overlap {
                break;
            }
        }

        if !found_overlap {
            self.create(start_addr, size, perm)
        }
    }

    pub fn create(&mut self, start_addr: usize, size: usize, perm: PagePermission) {
        assert!(start_addr & 0xfff == 0);
        assert!(size & 0xfff == 0);

        for reg in self.regions.iter() {
            if (reg.address >= start_addr && reg.address <= start_addr + size)
                || (reg.address + reg.size > start_addr
                    && reg.address + reg.size <= start_addr + size)
            {
                panic!(
                    "Overlapping regions! {:x} {:x} {:x} {:x}",
                    reg.address, reg.size, start_addr, size
                );
            }
        }

        map_region(&mut self.page_table, start_addr, size, perm);

        self.regions.push(Block {
            address: start_addr,
            size: size,
            permissions: perm,
        })
    }

    pub fn expand(&mut self, start_addr: usize, new_size: usize) {
        for r in &mut self.regions {
            if r.address == start_addr {
                // etc
                // TODO: page coalescing, etc.
                // For now, dumb ass 4k pages.

                if r.size > new_size {
                    // Wtf are you doing trying to shrink?
                    panic!("Stop it! expand called with smaller size");
                }

                unsafe {
                    for offset in (r.size..new_size).step_by(0x1000) {
                        let page = phys_allocator::alloc().unwrap();
                        self.page_table.map_4k(
                            page,
                            r.address + offset,
                            r.permissions,
                            MapType::NormalCachable,
                        );
                    }
                }

                r.size = new_size;
                return;
            }
        }
        panic!("Wtf?");
    }

    pub fn make_active(&self) {
        unsafe {
            arch::mmu::switch_to_page_table(self.page_table_phys);
        }
    }
}
