#![no_std]

pub trait InterruptController {
    fn init(&mut self);
    fn enable_interrupt(&mut self, n: u32);
    fn disable_interrupt(&mut self, n: u32);
    fn ack_interrupt(&mut self, n: u32);
}

pub trait Timer {
    fn init(&mut self);
    fn tick(&mut self);
    fn set_period_us(&mut self, n: u64);
    fn reset_timer(&mut self);
    fn enable_timer(&mut self);

    fn get_counter_ns(&self) -> u64;
}

pub trait SerialPort {
    fn read_byte(&mut self) -> u8;
    fn write_byte(&mut self, byte: u8);

    fn write_string(&mut self, a: &str) {
        for c in a.chars() {
            self.write_byte(c as u8);
        }
    }

    fn write_bytes(&mut self, a: &[u8]) {
        for c in a {
            self.write_byte(*c);
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub mod pc_uart;
#[cfg(target_arch = "x86_64")]
pub mod pic_interrupt_controller;
#[cfg(target_arch = "x86_64")]
pub mod pit_timer;
pub mod pl011_uart;

pub mod bcm_interrupt;

pub mod print;
