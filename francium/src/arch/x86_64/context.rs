use core::fmt::{Debug, Formatter};
use core::format_args;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct X86Regs {
    pub rax: usize,
    pub rbx: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rbp: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,

    pub interrupt_number: usize,
    pub error_code: usize,

    pub rip: usize,
    pub cs: usize,
    pub eflags: usize,
    pub rsp: usize,
    pub ss: usize,

    // End stack frame //
    pub fs: usize,
}

impl Debug for X86Regs {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.write_fmt(format_args!(
            "rax: {:016x}
rbx: {:016x}
rcx: {:016x}
rdx: {:016x}
rbp: {:016x}
rsi: {:016x}
rdi: {:016x}
r8:  {:016x}
r9:  {:016x}
r10: {:016x}
r11: {:016x}
r12: {:016x}
r13: {:016x}
r14: {:0016x}
r15: {:016x}

cs:rip: {:02x}:{:016x}
eflags: {:08x}
ss:rsp: {:02x}:{:016x}

fs: {:x}",
            self.rax,
            self.rbx,
            self.rcx,
            self.rdx,
            self.rbp,
            self.rsi,
            self.rdi,
            self.r8,
            self.r9,
            self.r10,
            self.r11,
            self.r12,
            self.r13,
            self.r14,
            self.r15,
            self.cs,
            self.rip,
            self.eflags,
            self.ss,
            self.rsp,
            self.fs
        ))
    }
}

impl X86Regs {
    const fn new() -> X86Regs {
        X86Regs {
            rip: 0,
            cs: 0,
            eflags: 0,
            rsp: 0,
            ss: 0,

            fs: 0,

            interrupt_number: 0,
            error_code: 0,

            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rbp: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ThreadContext {
    // regs includes kernel sp
    pub regs: X86Regs,
}

#[repr(C)]
pub struct ExceptionContext {
    // regs includes usermode sp
    pub regs: X86Regs,
}

impl ExceptionContext {
    pub const fn new() -> ExceptionContext {
        ExceptionContext {
            regs: X86Regs::new(),
        }
    }
}

impl ThreadContext {
    pub const fn new() -> ThreadContext {
        ThreadContext {
            regs: X86Regs::new(),
        }
    }
}
