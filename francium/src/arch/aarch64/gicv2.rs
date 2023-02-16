use crate::drivers::InterruptController;

pub struct GICv2 {
    gicd_base: usize,
    gicc_base: usize,
}

impl GICv2 {
    pub fn new(gicd_base: usize, gicc_base: usize) -> GICv2 {
        GICv2 {
            gicd_base: gicd_base,
            gicc_base: gicc_base,
        }
    }
}

// Distributor registers
const GICD_CTLR: usize = 0;
const GICD_ISENABLER: usize = 0x100;
const GICD_ICENABLER: usize = 0x180;
const GICD_ICPENDR: usize = 0x280;

// CPU interface Controller
const GICC_CTLR: usize = 0x00;
const GICC_PMR: usize = 0x04;
//const GICC_BPR: *mut u32 = (GICC_BASE + 0x0008) as *mut u32;

const GICD_ISENABLER_SIZE: u32 = 32;
const GICD_ICENABLER_SIZE: u32 = 32;
const GICD_ICPENDR_SIZE: u32 = 32;

impl InterruptController for GICv2 {
    fn init(&mut self) {
        unsafe {
            ((self.gicd_base + GICD_CTLR) as *mut u32).write_volatile(1);
            ((self.gicc_base + GICC_CTLR) as *mut u32).write_volatile(1);
            ((self.gicc_base + GICC_PMR) as *mut u32).write_volatile(0xff);
        }
    }

    fn enable_interrupt(&mut self, interrupt: u32) {
        unsafe {
            ((self.gicd_base + GICD_ISENABLER) as *mut u32)
                .add((interrupt / GICD_ISENABLER_SIZE) as usize)
                .write_volatile(1 << (interrupt % GICD_ISENABLER_SIZE));
        }
    }

    fn disable_interrupt(&mut self, interrupt: u32) {
        unsafe {
            ((self.gicd_base + GICD_ICENABLER) as *mut u32)
                .add((interrupt / GICD_ISENABLER_SIZE) as usize)
                .write_volatile(1 << (interrupt % GICD_ICENABLER_SIZE));
        }
    }

    fn ack_interrupt(&mut self, interrupt: u32) {
        unsafe {
            ((self.gicd_base + GICD_ICPENDR) as *mut u32)
                .add((interrupt / GICD_ICPENDR_SIZE) as usize)
                .write_volatile(1 << (interrupt % GICD_ICPENDR_SIZE));
        }
    }
}
