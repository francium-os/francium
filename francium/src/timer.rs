
struct Timer {
	// Fn<>? 
}

lazy_static! {
	static mut CURRENT_TIME: usize = 0;
	static ref TIMER_QUEUE: Mutex<Vec<Timer>> = Mutex::new(Scheduler::new());
}