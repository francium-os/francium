use crate::drivers::Timer;

use tock_registers::interfaces::{Readable, Writeable};
use aarch64_cpu::{registers::*};

pub struct ArchTimer {}

impl ArchTimer {
    pub fn new() -> ArchTimer {
        ArchTimer {}
    }
}

static mut TIMER_VAL: u64 = 0;

impl Timer for ArchTimer {
    fn init(&mut self) {}

    fn tick(&mut self) {}

    fn set_period_us(&mut self, us: u64) {
        // TODO: properly do this somehow, or at least with more precision
        unsafe {
            TIMER_VAL = (CNTFRQ_EL0.get() / 1000000) * us;
        }
    }

    fn reset_timer(&mut self) {
        unsafe {
            CNTP_TVAL_EL0.set(TIMER_VAL);
        }
    }

    fn enable_timer(&mut self) {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    }

    fn get_counter_ns(&self) -> u64 {
        let val = CNTPCT_EL0.get();

        // TODO: This is okay, but it feels a bit wrong.
        // 2**64 / (1000000000) = 18446744073
        // > 18446744073 seconds in years
        // approx. 584.5545 tropicalyear (time

        (val * 1000000000) / CNTFRQ_EL0.get()
    }
}
