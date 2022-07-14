#[repr(C, packed)]
struct GDTR {
	limit: u16,
	base: u64
}

#[repr(C, packed)]
struct GDTEntry {
	limit_low: u16,
	base_low: u16,
	base_middle: u8,
	access: u8,
	flags_limit: u8,
	base_high: u8
}

impl GDTEntry {
	const fn new(base: u32, limit: u32, access: u8, long_mode: bool) -> GDTEntry {
		// gran: 1, db: 1
		let mut flags: u8 = (1<<3);
		if long_mode {
			flags |= 1<<1;
		} else {
			flags |= 1<<2;
		}
		GDTEntry {
			limit_low: (limit & 0xffff) as u16,
			base_low: (base & 0xffff) as u16,
			base_middle: ((base & 0xff0000) >> 16) as u8,
			access: access,
			flags_limit: ((limit & 0xf0000) >> 16) as u8 | (flags << 4),
			base_high: ((base & 0xff000000) >> 24) as u8
		}
	}
}

const GDT_ENTRIES: [GDTEntry; 5] = [
	GDTEntry {
		limit_low: 0,
		base_low: 0,
		base_middle: 0,
		access: 0,
		flags_limit: 0,
		base_high: 0
	},

	// XXX: make proper bitfields!!

	// Kernel code
	GDTEntry::new(0, 0xfffff, (1<<7) | (1<<4) | (1<<3) | (1<<1), true),
	// Kernel data
	GDTEntry::new(0, 0xfffff, (1<<7) | (1<<4) | (1<<1), false),

	// User code
	GDTEntry::new(0, 0xfffff, (1<<7) | (3<<5) | (1<<4) | (1<<3) | (1<<1), true),
	// User data
	GDTEntry::new(0, 0xfffff, (1<<7) | (3<<5) | (1<<4) | (1<<1), false)
];

pub fn setup_gdt() {
	let gdtr = GDTR {
		limit: (GDT_ENTRIES.len()*8 - 1) as u16,
		base:  &GDT_ENTRIES as *const GDTEntry as u64
	};

	let gdtr_loc = &gdtr as *const GDTR as usize;
	unsafe {
		core::arch::asm!("lgdt [{gdtr_loc}]", gdtr_loc = in (reg) (gdtr_loc));
	}
}