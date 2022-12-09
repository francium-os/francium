use spin::Mutex;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::scheduler;
use crate::platform::DEFAULT_TIMER;
use crate::drivers::Timer;

struct TimerEntry {
	deadline: u64,
	callback: Box<dyn Fn() -> () + Send>
}

impl Drop for TimerEntry {
	fn drop(&mut self) {
		(self.callback)()
	}
}

lazy_static! {
	static ref TIMER_QUEUE: Mutex<Vec<TimerEntry>> = Mutex::new(Vec::new());
}

pub fn init() {
	let _locked = DEFAULT_TIMER.lock();
	// uhhhhhh idk
}

pub fn tick() {
	scheduler::tick();

	let current_time = { DEFAULT_TIMER.lock().get_counter_ns() };

	let mut queue_lock = TIMER_QUEUE.lock();
	queue_lock.retain(|x| x.deadline > current_time);
}

pub fn register_timer(offset: u64, callback: Box<dyn Fn() -> () + Send>) {
	let current_time = { DEFAULT_TIMER.lock().get_counter_ns() };
	let mut queue_lock = TIMER_QUEUE.lock();
	queue_lock.push(TimerEntry { deadline: current_time + offset, callback: callback });
}
