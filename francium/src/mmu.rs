extern crate alloc;
use alloc::boxed::Box;
use numtoa::NumToA;
use crate::write_uart;

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

		const ATTR_AP_2 = 1 << 7;
		const ATTR_AP_1 = 1 << 6;

		const ATTR_ACCESS = 1 << 10;
		// TODO: uhh, attributes ig
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

	fn addr(&self) -> u64 {
		// Extract bits 47:12
		self.entry & 0x000f_ffff_ffff_f000
	}

	fn flags(&self) -> EntryFlags {
		EntryFlags::from_bits_truncate(self.entry & !0x000f_ffff_ffff_f000)
	}

	fn set_flags(&mut self, flags: EntryFlags) {
		self.entry = self.addr() | flags.bits()
	}

	fn set_addr(&mut self, addr: usize) {
		// TODO TODO
		//assert!(addr.is_aligned(4096));

		self.entry = (addr & 0x000f_ffff_ffff_f000) as u64 | self.flags().bits();
	}
}

// https://9net.org/screenshots/1623169629.png

// for 4k granule, table holds 2**(log2(4096) - 3) = 512 entries
// table resolves 9 bits of address per level.

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; 512],
    tables: [Option<Box<PageTable>>; 512]
}

impl PageTable {
	pub fn new() -> PageTable {
		// TODO: weird
		const N: Option<Box<PageTable>> = None;
		PageTable {
			entries: [PageTableEntry::new(); 512],
			tables: [N; 512]
		}
	}

	// TODO PhysAddr newtype
	pub fn map_4k(&mut self, phys: usize, virt: usize) {
		let mut entry = PageTableEntry::new();
		// i think i can not care about flags wtf???
		entry.set_flags(EntryFlags::VALID | EntryFlags::TYPE_PAGE | EntryFlags::ATTR_ACCESS);
		entry.set_addr(phys);

		self.map_4k_internal(virt, entry, 0);
	}

	fn map_4k_internal(&mut self, virt: usize, entry: PageTableEntry, level: i32) {
		let off = (3-level) * 9 + 12;

		let index = (virt & (0x1ff << off)) >> off;
		if level < 3 {
			match &mut self.tables[index] {
				None => {
					let new_table = PageTable::new();
					let mut new_table_box = Box::new(new_table);

					let mut new_entry = PageTableEntry::new();
					new_entry.set_flags(EntryFlags::VALID | EntryFlags::TYPE_TABLE);
					new_entry.set_addr(new_table_box.as_mut() as *mut PageTable as usize); // uhh

					self.entries[index] = new_entry;
					new_table_box.as_mut().map_4k_internal(virt, entry, level + 1);

					let new_table_option = Some(new_table_box);
					self.tables[index] = new_table_option;
				},
				Some(x) => {
					x.map_4k_internal(virt, entry, level + 1);
				}
			}

		} else {
			// We are the final table! good.
			self.entries[index] = entry;
		}
	}

	/*fn map_2mb() {

	}

	fn map_1gb() {

	}*/
}

extern "C" {
	fn set_ttbr0_el1(ttbr: usize);
	fn get_sctlr_el1() -> usize;
	fn set_sctlr_el1(sctlr: usize);

	fn get_tcr_el1() -> usize;
	fn set_tcr_el1(tcr: usize);
}

pub fn enable_mmu(page_table: &PageTable) {
	// set ttbr0_el1
	unsafe {
		set_ttbr0_el1(page_table as *const PageTable as usize);

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

		let old_tcr = get_tcr_el1();

		/*let mut buffer = [0u8; 20];
		write_uart("tcr: ");
		write_uart(old_tcr.numtoa_str(16, &mut buffer));*/

		const TCR_IPS_48_BIT: usize = 0b101 << 32;
		const TCR_TG1_GRANULE_4K: usize = 0 << 30;
		const TCR_TG0_GRANULE_4K: usize = 0 << 14;

		const TCR_T0SZ_48_BIT: usize = 16;
		const TCR_T1SZ_48_BIT: usize = 16;

		let tcr = TCR_IPS_48_BIT | TCR_TG0_GRANULE_4K | TCR_TG1_GRANULE_4K | TCR_T0SZ_48_BIT | TCR_T1SZ_48_BIT;
		set_tcr_el1(tcr);

		// RES1 bits
		let mut sctlr = SCTLR_LSMAOE | SCTLR_NTLSMD | SCTLR_TSCXT;
		sctlr |= SCTLR_I | SCTLR_SPAN | SCTLR_C | SCTLR_M;
		set_sctlr_el1(sctlr);
	}
}