use spin::Mutex;
use alloc::vec::Vec;
use crate::scheduler;

struct Timer {
	// Fn<>? 
}

//static mut CURRENT_TIME: usize = 0;
lazy_static! {
	static ref TIMER_QUEUE: Mutex<Vec<Timer>> = Mutex::new(Vec::new());
}

pub fn init() {
	// Time begins now!
}

pub fn tick() {
	// XXX: draw the rest of the timer system
	scheduler::tick();
}