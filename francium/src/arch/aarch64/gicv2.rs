use core::ptr;
use crate::constants;
// VIRT_GIC_DIST = 0x08000000
// VIRT_GIC_CPU = 0x08010000

const GICD_BASE: usize = constants::PHYSMAP_BASE + 0x08000000;
const GICC_BASE: usize = constants::PHYSMAP_BASE + 0x08010000;

// Distributor registors
const GICD_CTLR: *mut u32 = GICD_BASE as *mut u32;
const GICD_ISENABLER: *mut u32 = (GICD_BASE + 0x0100) as *mut u32;
const GICD_ICPENDR: *mut u32 = (GICD_BASE + 0x280) as *mut u32;

// CPU interface Controller
const GICC_CTLR: *mut u32 = GICC_BASE as *mut u32;
const GICC_PMR: *mut u32 = (GICC_BASE + 0x0004) as *mut u32;
//const GICC_BPR: *mut u32 = (GICC_BASE + 0x0008) as *mut u32;

pub fn init() {
    unsafe {
        ptr::write_volatile(GICD_CTLR, 1);
        ptr::write_volatile(GICC_CTLR, 1);
        ptr::write_volatile(GICC_PMR, 0xff);
    }
}

const GICD_ISENABLER_SIZE: u32 = 32;
pub fn enable(interrupt: u32) {
    unsafe {
        ptr::write_volatile(
            GICD_ISENABLER.add((interrupt / GICD_ISENABLER_SIZE) as usize),
            1 << (interrupt % GICD_ISENABLER_SIZE)
        );
    }
}

const GICD_ICPENDR_SIZE: u32 = 32;
pub fn clear(interrupt: u32) {
    unsafe {
        ptr::write_volatile(
            GICD_ICPENDR.add((interrupt / GICD_ICPENDR_SIZE) as usize),
            1 << (interrupt % GICD_ICPENDR_SIZE)
        );
    }
}