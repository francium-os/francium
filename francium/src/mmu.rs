extern crate alloc;

use core::convert::TryFrom;

use crate::phys_allocator;
use crate::constants::*;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
	entry: u64
}

#[cfg(target_arch = "aarch64")]
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

#[cfg(target_arch = "x86_64")]
bitflags! {
	struct EntryFlags: u64 {
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


bitflags! {
	pub struct PagePermission : u64 {
		const READ_ONLY = 0;
		const WRITE = 1;
		const EXECUTE = 2;
		const KERNEL = 4;

		const USER_READ_ONLY = Self::READ_ONLY.bits;
		const USER_READ_WRITE = Self::READ_ONLY.bits | Self::WRITE.bits;
		const USER_READ_EXECUTE = Self::READ_ONLY.bits | Self::EXECUTE.bits;
		const USER_RWX = Self::READ_ONLY.bits | Self::WRITE.bits | Self::EXECUTE.bits;

		const KERNEL_READ_ONLY = Self::READ_ONLY.bits | Self::KERNEL.bits;
		const KERNEL_READ_WRITE = Self::READ_ONLY.bits | Self::WRITE.bits | Self::KERNEL.bits;
		const KERNEL_READ_EXECUTE = Self::READ_ONLY.bits| Self::EXECUTE.bits | Self::KERNEL.bits;
		const KERNEL_RWX = Self::KERNEL_READ_EXECUTE.bits | Self::WRITE.bits; 
	}
}

pub enum MapType {
	NormalCachable,
	NormalUncachable,
	Device
}

impl PageTableEntry {
	const fn new() -> PageTableEntry {
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

#[derive(Debug)]
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

// Big hack time!

// https://9net.org/screenshots/1627760764.png
#[cfg(target_arch = "aarch64")]
fn map_perms(perm: PagePermission) -> EntryFlags {
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

	flags
}

#[cfg(target_arch = "x86_64")]
fn map_perms(perm: PagePermission) -> EntryFlags {
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

	flags
}

#[cfg(target_arch = "aarch64")]
fn map_type(ty: MapType) -> EntryFlags {
	match ty {
		MapType::NormalCachable => EntryFlags::ATTR_INDEX_0,
		MapType::NormalUncachable => EntryFlags::ATTR_INDEX_1,
		MapType::Device => EntryFlags::ATTR_INDEX_2
	}
}

#[cfg(target_arch = "x86_64")]
fn map_type(ty: MapType) -> EntryFlags {
	match ty {
		MapType::NormalCachable => EntryFlags::empty(),
		MapType::NormalUncachable => EntryFlags::UNCACHEABLE,
		MapType::Device => EntryFlags::UNCACHEABLE // ???
	}
}

#[cfg(target_arch = "aarch64")]
fn get_default_flags() -> EntryFlags {
	EntryFlags::VALID | EntryFlags::TYPE_PAGE | EntryFlags::ATTR_ACCESS
}

#[cfg(target_arch = "x86_64")]
fn get_default_flags() -> EntryFlags {
	EntryFlags::VALID |  EntryFlags::ACCESS
}

#[cfg(target_arch = "aarch64")]
fn get_table_default_flags() -> EntryFlags {
	EntryFlags::VALID | EntryFlags::TYPE_TABLE
}

#[cfg(target_arch = "x86_64")]
fn get_table_default_flags() -> EntryFlags {
	EntryFlags::VALID | EntryFlags::TYPE_TABLE | EntryFlags::WRITABLE
}

#[cfg(target_arch = "aarch64")]
fn get_block_default_flags() -> EntryFlags {
	EntryFlags::VALID | EntryFlags::TYPE_BLOCK| EntryFlags::ATTR_ACCESS
}

#[cfg(target_arch = "x86_64")]
fn get_block_default_flags() -> EntryFlags {
	EntryFlags::VALID | EntryFlags::TYPE_BLOCK | EntryFlags::ACCESS
}



impl PageTable {
	pub const fn new() -> PageTable {
		PageTable {
			entries: [PageTableEntry::new(); 512],
		}
	}

	pub fn user_process(&self) -> PageTable {
		// TODO: is there a better way to do this

		let mut pg = PageTable::new();
		pg.entries[508] = self.entries[508];
		pg.entries[509] = self.entries[509];
		pg.entries[510] = self.entries[510];
		pg.entries[511] = self.entries[511];

		pg
	}

	pub fn map_4k(&mut self, phys: PhysAddr, virt: usize, perm: PagePermission, ty: MapType) {
		let mut entry = PageTableEntry::new();

		entry.set_flags(get_default_flags() | map_perms(perm) | map_type(ty));
		entry.set_addr(phys);

		unsafe {
			match self.map_internal(virt, entry, 0, 3) {
				Some(_) => (),
				None => {
					panic!("4k map failed!");
				}
			}
		}
	}

	pub fn map_2mb(&mut self, phys: PhysAddr, virt: usize, perm: PagePermission, ty: MapType) {
		assert!(phys.is_aligned(0x200000));
		assert!((virt & (0x200000-1)) == 0);

		let mut entry = PageTableEntry::new();

		entry.set_flags(get_block_default_flags() | map_perms(perm) | map_type(ty));
		entry.set_addr(phys);

		unsafe {
			match self.map_internal(virt, entry, 0, 2) {
				Some(_) => (),
				None => {
					panic!("2mb map failed!");
				}
			}
		}
	}

	pub fn map_1gb(&mut self, phys: PhysAddr, virt: usize, perm: PagePermission, ty: MapType) {
		assert!(phys.is_aligned(0x40000000));
		assert!((virt & (0x40000000-1)) == 0);
		let mut entry = PageTableEntry::new();

		entry.set_flags(get_block_default_flags() | map_perms(perm) | map_type(ty));
		entry.set_addr(phys);

		unsafe {
			match self.map_internal(virt, entry, 0, 1) {
				Some(_) => (),
				None => { 
					panic!("1gb map failed!");
				}
			}
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
				new_entry.set_flags(get_table_default_flags());
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

	unsafe fn walk_internal(&self, virt: usize, level: usize) -> Option<PhysAddr> {
		let final_level = 3;
		let off = (3-level) * 9 + 12;

		let index = (virt & (0x1ff << off)) >> off;

		// either block (done) or table (not done), or page (done)
		let entry_flags = self.entries[index].flags();
		if entry_flags.contains(EntryFlags::VALID) {
			if entry_flags.contains(EntryFlags::TYPE_TABLE) {
				if level < final_level {
					let x: usize = phys_to_virt(self.entries[index].addr());
					let page_table = x as *const PageTable;
					page_table.as_ref()?.walk_internal(virt, level + 1)
				}
				else {
					// calc block size from level
					let page_addr = self.entries[index].addr().0;
					let page_mask = (1<<(off)) - 1;
					Some(PhysAddr((virt & page_mask) + page_addr))
				}
			} else {
				// done, unless we are in the final level
				if level < final_level {
					let block_addr = self.entries[index].addr().0;
					let block_mask = (1<<(off)) - 1;
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
		unsafe {
			self.walk_internal(virt, 0)
		}
	}
}


pub fn phys_to_virt(phys: PhysAddr) -> usize {
	phys.0 + PHYSMAP_BASE
}

use crate::arch::mmu;
pub fn enable_mmu() {
	mmu::enable_mmu();
}
