pub trait InterruptController {
	fn init(&self);
	fn enable_interrupt(&self, n: u32);
	fn disable_interrupt(&self, n: u32);
	fn ack_interrupt(&self, n: u32);
}

pub trait Timer {
	fn init(&self);
	fn set_frequency_us(&self, n: u64);
	fn reset_timer(&self);
	fn enable_timer(&self);
}

pub mod pl011_uart;
