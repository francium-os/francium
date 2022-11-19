use spin::Mutex;
use alloc::vec::Vec;
use crate::scheduler;
use crate::platform::DEFAULT_TIMER;

struct Timer {
}

lazy_static! {
	static ref TIMER_QUEUE: Mutex<Vec<Timer>> = Mutex::new(Vec::new());
}

pub fn init() {
	let _locked = DEFAULT_TIMER.lock();
}

pub fn tick() {
	// XXX: draw the rest of the timer system
	scheduler::tick();
}
