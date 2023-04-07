use crate::InterruptController;
use francium_x86::io_port::{inb, io_wait, outb};

const PIC1_COMMAND: u16 = 0x20;
const PIC2_COMMAND: u16 = 0xa0;

const PIC1_DATA: u16 = PIC1_COMMAND + 1;
const PIC2_DATA: u16 = PIC2_COMMAND + 1;

const PIC_CMD_EOI: u8 = 0x20;

const ICW1_ICW4: u8 = 0x01; /* ICW4 (not) needed */
//const ICW1_SINGLE: u8 = 0x02; /* Single (cascade) mode */
//const ICW1_INTERVAL4: u8 = 0x04; /* Call address interval 4 (8) */
//const ICW1_LEVEL: u8 = 0x08; /* Level triggered (edge) mode */
const ICW1_INIT: u8 = 0x10; /* Initialization - required! */

const ICW4_8086: u8 = 0x01; /* 8086/88 (MCS-80/85) mode */
//const ICW4_AUTO: u8 = 0x02; /* Auto (normal) EOI */
//const ICW4_BUF_SLAVE: u8 = 0x08; /* Buffered mode/slave */
//const ICW4_BUF_MASTER: u8 = 0x0C; /* Buffered mode/master */
//const ICW4_SFNM: u8 = 0x10; /* Special fully nested (not) */
fn pic_send_eoi(interrupt: u8) {
    if interrupt >= 8 {
        outb(PIC2_COMMAND, PIC_CMD_EOI);
    }
    outb(PIC1_COMMAND, PIC_CMD_EOI);
}

pub struct PIC {}

impl PIC {
    pub fn new() -> PIC {
        PIC {}
    }
}

impl InterruptController for PIC {
    fn init(&mut self) {
        // Remap PICs.
        // Yes, this is stolen directly off OSDev.
        // https://wiki.osdev.org/PIC#Code_Examples

        let offset1 = 32;
        let offset2 = 32 + 8;

        let a1 = inb(PIC1_DATA);
        let a2 = inb(PIC2_DATA);

        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4); // starts the initialization sequence (in cascade mode)
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC1_DATA, offset1); // ICW2: Master PIC vector offset
        io_wait();
        outb(PIC2_DATA, offset2); // ICW2: Slave PIC vector offset
        io_wait();
        outb(PIC1_DATA, 4); // ICW3: tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
        io_wait();
        outb(PIC2_DATA, 2); // ICW3: tell Slave PIC its cascade identity (0000 0010)
        io_wait();

        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        outb(PIC1_DATA, a1); // restore saved masks.
        outb(PIC2_DATA, a2);
    }

    fn enable_interrupt(&mut self, n: u32) {
        if n < 8 {
            outb(PIC1_DATA, inb(PIC1_DATA) & !(1 << n));
        } else {
            outb(PIC2_DATA, inb(PIC2_DATA) & !(1 << (n - 8)));
        }
    }

    fn disable_interrupt(&mut self, n: u32) {
        if n < 8 {
            outb(PIC1_DATA, inb(PIC1_DATA) | 1 << n);
        } else {
            outb(PIC2_DATA, inb(PIC2_DATA) | 1 << (n - 8));
        }
    }

    fn ack_interrupt(&mut self, n: u32) {
        pic_send_eoi(n as u8);
    }

    const NUM_PENDING: u32 = 1;
    fn read_pending(&self, _n: u32) -> u32 {
        0
    }
}
