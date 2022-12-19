use crate::Timer;
use core::arch::asm;

pub struct PIT {
    reload_value: u16,
    period_ns: u64,
    counter: u64,
}

impl PIT {
    pub fn new() -> PIT {
        // etc
        PIT {
            counter: 0,
            period_ns: 0,
            reload_value: 0,
        }
    }
}

//const PIT_CHANNEL_0: u8 = 0 << 6;
//const PIT_CHANNEL_1: u8 = 1 << 6;
//const PIT_CHANNEL_2: u8 = 2 << 6;

//const PIT_ACCESS_LATCH: u8 = 0 << 4;
//const PIT_ACCESS_LOW: u8 = 1 << 4;
//const PIT_ACCESS_HIGH: u8 = 2 << 4;
const PIT_ACCESS_BOTH: u8 = 3 << 4;

//const PIT_OP_MODE_0: u8 = 0 << 1;
//const PIT_OP_MODE_1: u8 = 1 << 1;
//const PIT_OP_MODE_2: u8 = 2 << 1;
const PIT_OP_MODE_3: u8 = 3 << 1;
//const PIT_OP_MODE_4: u8 = 4 << 1;
//const PIT_OP_MODE_5: u8 = 5 << 1;
// 6,7 are aliases of 2/3
//const PIT_BCD_MODE: u8 = 1;
const PIT_BINARY_MODE: u8 = 0;

fn write_mode_command_reg(channel: u8, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") 0x43, in("al") (channel << 6) | value);
    }
}

fn write_data_reg(channel: u8, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") (0x40 + channel) as u16, in("al") value);
    }
}

fn write_data_reg_u16(channel: u8, value: u16) {
    write_data_reg(channel, (value & 0xff) as u8);
    write_data_reg(channel, ((value & 0xff00) >> 8) as u8);
}

impl Timer for PIT {
    fn init(&mut self) {}

    fn tick(&mut self) {
        self.counter += self.period_ns;
    }

    fn set_period_us(&mut self, us: u64) {
        // PIT ticks at 1.193182 mhz
        self.period_ns = us * 1000;
        self.reload_value = ((us * 1193182) / 1000000) as u16;
    }

    fn reset_timer(&mut self) {
        write_mode_command_reg(0, PIT_ACCESS_BOTH | PIT_OP_MODE_3 | PIT_BINARY_MODE);
        write_data_reg_u16(0, self.reload_value & 0xfffe); // Mask out the 1 bit.
    }

    fn enable_timer(&mut self) {
        // can we even, do this
        self.reset_timer();
    }

    fn get_counter_ns(&self) -> u64 {
        self.counter
    }
}
