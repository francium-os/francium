use crate::arch::x86_64::msr;
use core::arch::asm;

pub unsafe fn setup_per_cpu(per_cpu_base: usize) {
    // We need this to be visible immediately.
    // The kernel should swapgs on exit, so this is ok.

    msr::write_gs_base(per_cpu_base);
}

pub unsafe fn get_per_cpu_base() -> usize {
    let base: usize;
    asm!("mov {base}, gs:0", base = out(reg)(base));
    base
}
