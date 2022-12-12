pub trait InterruptController {
    fn init(&self);
    fn enable_interrupt(&self, n: u32);
    fn disable_interrupt(&self, n: u32);
    fn ack_interrupt(&self, n: u32);
}

pub trait Timer {
    fn init(&mut self);
    fn tick(&mut self);
    fn set_period_us(&mut self, n: u64);
    fn reset_timer(&mut self);
    fn enable_timer(&mut self);

    fn get_counter_ns(&self) -> u64;
}

#[cfg(target_arch = "x86_64")]
pub mod pc_uart;
#[cfg(target_arch = "x86_64")]
pub mod pic_interrupt_controller;
#[cfg(target_arch = "x86_64")]
pub mod pit_timer;
pub mod pl011_uart;
