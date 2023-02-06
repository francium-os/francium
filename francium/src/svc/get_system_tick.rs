use crate::timer;

pub fn svc_get_system_tick() -> u64 {
    timer::get_counter_ns()
}
