use francium_x86::idt::{IDTEntry, use_idt};

const NULL_IDT: IDTEntry = IDTEntry::null();
static mut IDT_ENTRIES: [IDTEntry; 48] = [NULL_IDT; 48];
use crate::arch::x86_64::interrupt_handlers::INTERRUPT_HANDLERS;

pub fn setup_idt() {
    unsafe {
        for i in 0..IDT_ENTRIES.len() {
            IDT_ENTRIES[i] = IDTEntry::new(INTERRUPT_HANDLERS[i] as usize, 0, 0);
        }
        use_idt(&IDT_ENTRIES);
    }
}
