use core::arch::asm;
use crate::drivers::Timer;

pub struct PITTimer {}

impl PITTimer {
	pub fn new() -> PITTimer {
		// etc
		PITTimer{}
	}
}

impl Timer for PITTimer {
	fn init(&self) {}

    fn set_period_us(&self, us: u64) {
        unsafe {
           
        }
    }

    fn reset_timer(&self) {
        unsafe {
            
        }
    }

    fn enable_timer(&self) {
        unsafe {
            
        }
    }

    fn get_counter_ns(&self) -> u64 {
        // hm
        0
    }
}