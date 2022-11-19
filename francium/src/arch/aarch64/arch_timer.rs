use crate::drivers::Timer;

pub struct ArchTimer{}
impl ArchTimer {
    pub fn new() -> ArchTimer {
        ArchTimer{}
    }
}

static mut TIMER_VAL: u64 = 0;

extern "C" {
    fn get_cntfrq() -> u64;
    fn set_cntp_tval_el0(val: u64);
    fn set_cntp_ctl_el0(ctl: u64);
}

impl Timer for ArchTimer {
    fn init(&self) {}

    fn set_frequency_us(&self, us: u64) {
        unsafe {
            // TODO: properly do this somehow, or at least with more precision
            TIMER_VAL = (get_cntfrq()/1000000) * us;
        }
    }

    fn reset_timer(&self) {
        unsafe {
            set_cntp_tval_el0(TIMER_VAL);
        }
    }

    fn enable_timer(&self) {
        unsafe {
            set_cntp_ctl_el0(1);
        }
    }
}




