use core::arch::asm;
use crate::arch::x86_64::interrupt_handlers;

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
	const fn null() -> IDTEntry {
		IDTEntry {
			offset_low: 0,
			seg_selector: 0,
			ist: 0,
			type_flags: 0,
			offset_middle: 0,
			offset_high: 0,
			reserved: 0
		}
	}

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

const NULL_IDT: IDTEntry = IDTEntry::null();
static mut IDT_ENTRIES: [IDTEntry; 48] = [NULL_IDT; 48];
use crate::arch::x86_64::interrupt_handlers::INTERRUPT_HANDLERS;

pub fn setup_idt() {
	unsafe {
		for i in 0..IDT_ENTRIES.len() {
			//if i == 0xe {
			//	IDT_ENTRIES[i] = IDTEntry::new(INTERRUPT_HANDLERS[i] as usize, 1, 0);
			//} else {
			IDT_ENTRIES[i] = IDTEntry::new(INTERRUPT_HANDLERS[i] as usize, 0, 0);
			//}
		}

		let idtr = IDTR {
			limit: (IDT_ENTRIES.len()*16 - 1) as u16,
			base:  &IDT_ENTRIES as *const IDTEntry as u64
		};

		let idtr_loc = &idtr as *const IDTR as usize;

		asm!("lidt [{idtr_loc}]", idtr_loc = in (reg) (idtr_loc));
	}
}