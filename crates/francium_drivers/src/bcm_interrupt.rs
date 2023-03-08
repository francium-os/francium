use crate::InterruptController;

pub struct BCMInterrupt {
    base_address: usize,
}

impl BCMInterrupt {
    pub fn new(base_address: usize) -> BCMInterrupt {
        BCMInterrupt {
            base_address: base_address,
        }
    }

    /*unsafe fn read_pending_1(&mut self) -> u32 {
        const PENDING_1: usize = 0x204;
        ((self.base_address + PENDING_1) as *mut u32).read_volatile()
    }

    unsafe fn read_pending_2(&mut self) -> u32 {
        const PENDING_2: usize = 0x208;
        ((self.base_address + PENDING_2) as *mut u32).read_volatile()
    }*/

    unsafe fn write_pending_1(&mut self, val: u32) {
        const PENDING_1: usize = 0x204;
        ((self.base_address + PENDING_1) as *mut u32).write_volatile(val)
    }

    unsafe fn write_pending_2(&mut self, val: u32) {
        const PENDING_2: usize = 0x208;
        ((self.base_address + PENDING_2) as *mut u32).write_volatile(val)
    }

    unsafe fn _read_fiq_control(&mut self) -> u32 {
        const FIQ_CONTROL: usize = 0x20c;
        ((self.base_address + FIQ_CONTROL) as *mut u32).read_volatile()
    }

    unsafe fn _write_fiq_control(&mut self, val: u32) {
        const FIQ_CONTROL: usize = 0x20c;
        ((self.base_address + FIQ_CONTROL) as *mut u32).write_volatile(val)
    }

    unsafe fn write_enable_1(&mut self, val: u32) {
        const ENABLE_1: usize = 0x210;
        ((self.base_address + ENABLE_1) as *mut u32).write_volatile(val)
    }

    unsafe fn write_enable_2(&mut self, val: u32) {
        const ENABLE_2: usize = 0x214;
        ((self.base_address + ENABLE_2) as *mut u32).write_volatile(val)
    }

    unsafe fn write_disable_1(&mut self, val: u32) {
        const DISABLE_1: usize = 0x21c;
        ((self.base_address + DISABLE_1) as *mut u32).write_volatile(val)
    }

    unsafe fn write_disable_2(&mut self, val: u32) {
        const DISABLE_2: usize = 0x220;
        ((self.base_address + DISABLE_2) as *mut u32).write_volatile(val)
    }
}

impl InterruptController for BCMInterrupt {
    fn init(&mut self) {}

    fn enable_interrupt(&mut self, n: u32) {
        if n < 32 {
            unsafe {
                self.write_enable_1(1 << n);
            }
        } else {
            unsafe {
                self.write_enable_2(1 << (n - 32));
            }
        }
    }

    fn disable_interrupt(&mut self, n: u32) {
        if n < 32 {
            unsafe {
                self.write_disable_1(1 << n);
            }
        } else {
            unsafe { self.write_disable_2(1 << (n - 32)) }
        }
    }

    fn ack_interrupt(&mut self, n: u32) {
        if n < 32 {
            unsafe {
                self.write_pending_1(1 << n);
            }
        } else {
            unsafe { self.write_pending_2(1 << (n - 32)) }
        }
    }
}
