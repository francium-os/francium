static mut TIMER_VAL: usize = 0;

extern "C" {
    fn get_cntfrq() -> usize;
    fn set_cntp_tval_el0(val: usize);
    fn set_cntp_ctl_el0(ctl: usize);
}

pub fn set_frequency_us(us: usize) {
    unsafe {
        // TODO: properly do this somehow, or at least with more precision
        TIMER_VAL = (get_cntfrq()/1000000) * us;
    }
}

pub fn reset_timer() {
    unsafe {
        set_cntp_tval_el0(TIMER_VAL);
    }
}

pub fn enable() {
    unsafe {
        set_cntp_ctl_el0(1);
    }
}