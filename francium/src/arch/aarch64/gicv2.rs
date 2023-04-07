use crate::drivers::InterruptController;

pub struct GICv2 {
    gicd_base: usize,
    gicc_base: usize,
}


// XXX please god rewrite this using registers

// Distributor registers
const GICD_CTLR: usize = 0;
const GICD_ISENABLER: usize = 0x100;
const GICD_ICENABLER: usize = 0x180;
const GICD_ICPENDR: usize = 0x280;
const GICD_ICFGR: usize = 0xC00;

// CPU interface Controller
const GICC_CTLR: usize = 0x00;
const GICC_PMR: usize = 0x04;
const GICC_IAR: usize = 0x000C;
const GICC_EOIR: usize = 0x0010;
//const GICC_BPR: *mut u32 = (GICC_BASE + 0x0008) as *mut u32;

const GICD_ISENABLER_SIZE: u32 = 32;
const GICD_ICENABLER_SIZE: u32 = 32;
const GICD_ICPENDR_SIZE: u32 = 32;
const GICD_ICFGR_SIZE: u32 = 16;

impl GICv2 {
    pub fn new(gicd_base: usize, gicc_base: usize) -> GICv2 {
        GICv2 {
            gicd_base: gicd_base,
            gicc_base: gicc_base,
        }
    }

    fn set_config(&mut self, interrupt: u32, is_level_triggered: bool) {
        // XXX: hella broken

        let bit = if is_level_triggered { 0 } else { 2 };

        unsafe {
            let ptr = ((self.gicd_base + GICD_ICFGR) as *mut u32)
            .add((interrupt / GICD_ICFGR_SIZE) as usize);
            ptr.write_volatile(ptr.read_volatile() | (bit << ((interrupt % GICD_ISENABLER_SIZE) * 2)));
        }
    }
}

impl InterruptController for GICv2 {
    fn init(&mut self) {
        unsafe {
            ((self.gicd_base + GICD_CTLR) as *mut u32).write_volatile(1);
            ((self.gicc_base + GICC_CTLR) as *mut u32).write_volatile(1);
            ((self.gicc_base + GICC_PMR) as *mut u32).write_volatile(0xff);

            // HACK: Virt PCIe interrupts are level triggered
            self.set_config(35, true);
            self.set_config(35, true);
            self.set_config(36, true);
            self.set_config(37, true);
        }
    }

    fn enable_interrupt(&mut self, interrupt: u32) {
        println!("Enable interrupt: {}", interrupt);
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
            /*((self.gicd_base + GICD_ICPENDR) as *mut u32)
                .add((interrupt / GICD_ICPENDR_SIZE) as usize)
                .write_volatile(1 << (interrupt % GICD_ICPENDR_SIZE));*/

            ((self.gicc_base + GICC_EOIR) as *mut u32).write_volatile(interrupt)
        }
    }

    // TODO: Correct value?
    const NUM_PENDING: u32 = 2;
    fn read_pending(&self, i: u32) -> u32 {
        let bits = unsafe {
            ((self.gicd_base + GICD_ICPENDR) as *mut u32)
                .add(i as usize)
                .read_volatile()
        };
        bits
    }

    fn next_pending(&self) -> Option<u32> {
        let interrupt_num = unsafe { 
            ((self.gicc_base + GICC_IAR) as *mut u32).read_volatile() & 0x3ff
        };

        if interrupt_num == 1023 {
            None
        } else {
            Some(interrupt_num)
        }
    }
}
