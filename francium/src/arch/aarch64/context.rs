#[repr(C)]
#[derive(Debug, Clone)]
pub struct ThreadContext {
    // regs includes kernel sp
    pub regs: [usize; 32],
    pub neon: [[u8; 16]; 32],
}

#[repr(C)]
pub struct ExceptionContext {
    // regs includes usermode sp
    pub regs: [usize; 32],
    pub saved_pc: usize,
    pub saved_spsr: usize,
    pub saved_tpidr: usize,
}

impl ExceptionContext {
    pub const fn new() -> ExceptionContext {
        ExceptionContext {
            regs: [0; 32],
            saved_pc: 0,
            saved_spsr: 0,
            saved_tpidr: 0,
        }
    }
}

impl ThreadContext {
    pub const fn new() -> ThreadContext {
        ThreadContext {
            regs: [0; 32],
            neon: [[0; 16]; 32],
        }
    }
}
