#[repr(C, packed)]
struct IDTR {
	limit: u16,
	base: u64
}

#[repr(C, packed)]
struct IDTEntry {
	offset_low: u16,
	seg_selector: u16,
	ist: u8,
	type_flags: u8,
	offset_middle: u16,
	offset_high: u32,
	reserved: u32
}

impl IDTEntry {
	const fn new(offset: usize, ist: u8, flags: u8) -> IDTEntry {
		// cheat a little: hardcode kernel code seg
		IDTEntry {
			offset_low: (offset & 0xffff) as u16,
			seg_selector: 8,
			ist: ist,
			type_flags: (1<<7) | (0b1110),
			offset_middle: ((offset & 0xffff0000) >> 16) as u16,
			offset_high: ((offset & 0xffffffff00000000) >> 32) as u32,
			reserved: 0
		}
	}
}

const IDT_ENTRIES: [IDTEntry; 4] = [
	IDTEntry::new(interrupt_3, 0, 0),
	IDTEntry::new(idt_2, 0, 0),
	IDTEntry::new(idt_3, 0, 0),
	IDTEntry::new(idt_4, 0, 0)
];

pub fn setup_idt() {
	let idtr = IDTR {
		limit: (IDT_ENTRIES.len()*16 - 1) as u16,
		base:  &IDT_ENTRIES as *const IDTEntry as u64
	};

	let idtr_loc = &idtr as *const IDTR as usize;
	unsafe {
		core::arch::asm!("lidt [{idtr_loc}]", idtr_loc = in (reg) (idtr_loc));
	}
}