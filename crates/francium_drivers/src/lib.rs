#![no_std]

// ie gic core, local apic
pub trait InterruptController {
    fn init(&mut self);

    fn ack_interrupt(&mut self, n: u32);

    const NUM_PENDING: u32;
    fn read_pending(&self, i: u32) -> u32;
    fn next_pending(&self) -> Option<u32> {
        for i in 0..Self::NUM_PENDING {
            let bits = self.read_pending(i);
            let zeros = bits.leading_zeros();
            if zeros != 32 {
                return Some(32 - (zeros + 1) + i * 32);
            }
        }

        None
    }
}

// ie gic distributor, IOAPIC
pub trait InterruptDistributor {
    fn init(&mut self);
    fn enable_interrupt(&mut self, n: u32);
    fn disable_interrupt(&mut self, n: u32);
    // TODO: setup core affinity
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
pub mod pc_io_apic;
#[cfg(target_arch = "x86_64")]
pub mod pc_local_apic;
#[cfg(target_arch = "x86_64")]
pub mod pc_uart;
#[cfg(target_arch = "x86_64")]
pub mod pic_interrupt_controller;

#[cfg(target_arch = "x86_64")]
pub mod pit_timer;
pub mod pl011_uart;

pub mod bcm_interrupt;

pub mod print;
