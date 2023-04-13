use crate::InterruptController;

pub struct BCMGlobalInterrupt {
    base_address: usize,
}

impl BCMGlobalInterrupt {
    pub fn new(base_address: usize) -> BCMGlobalInterrupt {
        BCMGlobalInterrupt {
            base_address: base_address,
        }
    }

    unsafe fn read_pending_1(&self) -> u32 {
        const PENDING_1: usize = 0x204;
        ((self.base_address + PENDING_1) as *mut u32).read_volatile()
    }

    unsafe fn read_pending_2(&self) -> u32 {
        const PENDING_2: usize = 0x208;
        ((self.base_address + PENDING_2) as *mut u32).read_volatile()
    }

    unsafe fn write_pending_1(&mut self, val: u32) {
        const PENDING_1: usize = 0x204;
        ((self.base_address + PENDING_1) as *mut u32).write_volatile(val)
    }

    unsafe fn write_pending_2(&mut self, val: u32) {
        const PENDING_2: usize = 0x208;
        ((self.base_address + PENDING_2) as *mut u32).write_volatile(val)
    }

    unsafe fn _read_fiq_control(&self) -> u32 {
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

impl InterruptController for BCMGlobalInterrupt {
    fn init(&mut self) {}

    /*fn enable_interrupt(&mut self, n: u32) {
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
    }*/

    fn ack_interrupt(&mut self, n: u32) {
        if n < 32 {
            unsafe {
                self.write_pending_1(1 << n);
            }
        } else {
            unsafe { self.write_pending_2(1 << (n - 32)) }
        }
    }

    const NUM_PENDING: u32 = 2;
    fn read_pending(&self, i: u32) -> u32 {
        if i == 0 {
            unsafe { self.read_pending_1() }
        } else if i == 1 {
            unsafe { self.read_pending_2() }
        } else {
            0
        }
    }
}

pub struct BCMLocalInterrupt {
    base_address: usize
}

impl BCMLocalInterrupt {
    pub fn new(base_address: usize) -> BCMLocalInterrupt {
        BCMLocalInterrupt {
            base_address: base_address,
        }
    }
}

/*
Address: 0x4000_0040 Core 0 Timers interrupt control
Address: 0x4000_0044 Core 1 Timers interrupt control
Address: 0x4000_0048 Core 2 Timers interrupt control
Address: 0x4000_004C Core 3 Timers interrupt control

Address: 0x4000_0050 Core0 Mailboxes interrupt control
Address: 0x4000_0054 Core1 Mailboxes interrupt control
Address: 0x4000_0058 Core2 Mailboxes interrupt control
Address: 0x4000_005C Core3 Mailboxes interrupt control

Pending registers:

Address: 0x4000_0060 Core0 interrupt source
Address: 0x4000_0064 Core1 interrupt source
Address: 0x4000_0068 Core2 interrupt source
Address: 0x4000_006C Core3 interrupt source
*/

impl InterruptController for BCMLocalInterrupt {
    fn init(&mut self) {}

    fn ack_interrupt(&mut self, n: u32) {
        unimplemented!();
    }

    const NUM_PENDING: u32 = 1;
    fn read_pending(&self, i: u32) -> u32 {
        unimplemented!();
    }
}
