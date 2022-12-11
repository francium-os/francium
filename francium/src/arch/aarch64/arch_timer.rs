use crate::drivers::Timer;
use core::arch::asm;

pub struct ArchTimer {}

impl ArchTimer {
    pub fn new() -> ArchTimer {
        ArchTimer {}
    }
}

static mut TIMER_VAL: u64 = 0;

fn get_cntfrq() -> u64 {
    unsafe {
        let mut value;
        asm!("mrs {cntfrq_el0}, cntfrq_el0", cntfrq_el0 = out(reg) value);
        value
    }
}

/*unsafe fn get_cntp_tval_el0() -> usize {
    let mut value;
    asm!("mrs {cntp_ctl_el0}, cntp_ctl_el0", cntp_ctl_el0 = out(reg) value);
    value
}*/

unsafe fn set_cntp_tval_el0(value: u64) {
    asm!("msr cntp_tval_el0, {cntp_tval_el0}", cntp_tval_el0 = in(reg) value);
}

unsafe fn set_cntp_ctl_el0(value: u64) {
    asm!("msr cntp_ctl_el0, {cntp_ctl_el0}", cntp_ctl_el0 = in(reg) value);
}

impl Timer for ArchTimer {
    fn init(&mut self) {}

    fn set_period_us(&mut self, us: u64) {
        unsafe {
            // TODO: properly do this somehow, or at least with more precision
            TIMER_VAL = (get_cntfrq() / 1000000) * us;
        }
    }

    fn reset_timer(&mut self) {
        unsafe {
            set_cntp_tval_el0(TIMER_VAL);
        }
    }

    fn enable_timer(&mut self) {
        unsafe {
            set_cntp_ctl_el0(1);
        }
    }

    fn get_counter_ns(&self) -> u64 {
        let mut val: u64;
        unsafe {
            asm!("mrs {val}, CNTPCT_EL0", val = out(reg)(val));
        }

        // TODO: This is okay, but it feels a bit wrong.
        // 2**64 / (1000000000) = 18446744073
        // > 18446744073 seconds in years
        // approx. 584.5545 tropicalyear (time

        (val * 1000000000) / get_cntfrq()
    }
}
