extern crate alloc;

use alloc::boxed::Box;
use core::convert::TryFrom;

use crate::phys_allocator;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PhysAddr(pub usize);

impl core::fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl PhysAddr {
	pub fn is_aligned(&self, n: usize) -> bool {
		self.0 & (n-1) == 0
	}
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
	entry: u64
}

bitflags! {
	struct EntryFlags: u64 {
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


// TODO TODO PhysAddr newtype

impl PageTableEntry {
	fn new() -> PageTableEntry {
		PageTableEntry { entry: 0 }
	}	

	fn addr(&self) -> PhysAddr {
		// Extract bits 47:12
		PhysAddr(usize::try_from(self.entry & 0x000f_ffff_ffff_f000).unwrap())
	}

	fn flags(&self) -> EntryFlags {
		EntryFlags::from_bits_truncate(self.entry & !0x000f_ffff_ffff_f000)
	}

	fn set_flags(&mut self, flags: EntryFlags) {
		self.entry = self.addr().0 as u64 | flags.bits()
	}

	fn set_addr(&mut self, addr: PhysAddr) {
		assert!(addr.is_aligned(4096));
		self.entry = (addr.0 as u64 & 0x000f_ffff_ffff_f000) | self.flags().bits();
	}
}

// https://9net.org/screenshots/1623169629.png

// for 4k granule, table holds 2**(log2(4096) - 3) = 512 entries
// table resolves 9 bits of address per level.

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

struct MappingError;

impl PageTable {
	pub fn new() -> PageTable {
		PageTable {
			entries: [PageTableEntry::new(); 512],
		}
	}

	pub fn map_4k(&mut self, phys: PhysAddr, virt: usize) {
		let mut entry = PageTableEntry::new();

		entry.set_flags(EntryFlags::VALID | EntryFlags::TYPE_PAGE | EntryFlags::ATTR_ACCESS);
		entry.set_addr(phys);

		unsafe {
			self.map_internal(virt, entry, 0, 3);
		}
	}

	pub fn map_2mb(&mut self, phys: PhysAddr, virt: usize) {
		assert!(phys.is_aligned(0x200000));
		assert!((virt & (0x200000-1)) == 0);

		let mut entry = PageTableEntry::new();

		entry.set_flags(EntryFlags::VALID | EntryFlags::TYPE_BLOCK | EntryFlags::ATTR_ACCESS);
		entry.set_addr(phys);

		unsafe {
			self.map_internal(virt, entry, 0, 2);
		}
	}

	pub fn map_1gb(&mut self, phys: PhysAddr, virt: usize) {
		assert!(phys.is_aligned(0x40000000));
		assert!((virt & (0x40000000-1)) == 0);
		let mut entry = PageTableEntry::new();

		entry.set_flags(EntryFlags::VALID | EntryFlags::TYPE_BLOCK | EntryFlags::ATTR_ACCESS);
		entry.set_addr(phys);

		unsafe {
			self.map_internal(virt, entry, 0, 1);
		}
	}

	unsafe fn map_internal(&mut self, virt: usize, entry: PageTableEntry, level: i32, final_level: i32) -> Option<()> {
		let off = (3-level) * 9 + 12;

		let index = (virt & (0x1ff << off)) >> off;
		if level < final_level {
			if self.entries[index].entry == 0 {
				let new_table_phys: PhysAddr = phys_allocator::alloc()?;
				let x: usize = phys_to_virt(new_table_phys);
				let page_table = x as *mut PageTable;
				*page_table = PageTable::new();

				let mut new_entry = PageTableEntry::new();
				new_entry.set_flags(EntryFlags::VALID | EntryFlags::TYPE_TABLE);
				new_entry.set_addr(new_table_phys); // uh
				self.entries[index] = new_entry;
			}

			let x: usize = phys_to_virt(self.entries[index].addr());
			let page_table = x as *mut PageTable;
			page_table.as_mut()?.map_internal(virt, entry, level + 1, final_level)
		} else {
			// We are the final table! good.
			self.entries[index] = entry;
			Some(())
		}
	}
}

extern "C" {
	fn set_ttbr0_el1(ttbr: PhysAddr);
	fn set_ttbr1_el1(ttbr: PhysAddr);
	fn get_sctlr_el1() -> usize;
	fn set_sctlr_el1(sctlr: usize);

	fn get_tcr_el1() -> usize;
	fn set_tcr_el1(tcr: usize);
}

pub fn phys_to_virt(phys: PhysAddr) -> usize {
	let physmap_base = 0xffffff0000000000;
	phys.0 + physmap_base
}

pub fn virt_to_phys(virt: usize) -> PhysAddr {
	let kernel_base = 0xfffffff800000000;
	let phys_base = 0x40000000;
	PhysAddr(virt - kernel_base + phys_base)
}

pub fn enable_mmu(page_table: &PageTable) {
	unsafe {
		set_ttbr0_el1(virt_to_phys(page_table as *const PageTable as usize));
		set_ttbr1_el1(virt_to_phys(page_table as *const PageTable as usize));

		// enable caches + mmu
		// enable sp alignment?

		const SCTLR_LSMAOE: usize = 1<<29;
		const SCTLR_NTLSMD: usize = 1<<28;
		const SCTLR_TSCXT: usize =  1<<20;
		//const SCTLR_ITD = 1<<7;

		const SCTLR_I: usize    = 1 << 12;
		const SCTLR_SPAN: usize = 1 << 3;
		const SCTLR_C: usize    = 1 << 2;
		const SCTLR_M: usize    = 1 << 0;

		const TCR_IPS_48_BIT: usize = 0b101 << 32;
		const TCR_TG1_GRANULE_4K: usize = 0 << 30;
		const TCR_TG0_GRANULE_4K: usize = 0 << 14;

		const TCR_T0SZ_48_BIT: usize = 16;
		const TCR_T1SZ_48_BIT: usize = 16 << 16;

		let tcr = TCR_IPS_48_BIT | TCR_TG0_GRANULE_4K | TCR_TG1_GRANULE_4K | TCR_T0SZ_48_BIT | TCR_T1SZ_48_BIT;
		set_tcr_el1(tcr);

		// RES1 bits
		let mut sctlr = SCTLR_LSMAOE | SCTLR_NTLSMD | SCTLR_TSCXT;

		// icache, dcache, sp alignment, mmu enable
		sctlr |= SCTLR_I | SCTLR_SPAN | SCTLR_C | SCTLR_M;
		set_sctlr_el1(sctlr);
	}
}