use crate::drivers::Timer;

use aarch64_cpu::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

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
        let ns_per_tick = (1000000000) / CNTFRQ_EL0.get();

        val * ns_per_tick
    }
}
