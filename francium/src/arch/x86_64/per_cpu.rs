use crate::arch::x86_64::msr;
use core::arch::asm;

pub unsafe fn setup_per_cpu(per_cpu_base: usize) {
    // We need this to be visible immediately.
    println!("==================== gs base {:x}", per_cpu_base);
    msr::write_gs_base(per_cpu_base);
}

pub unsafe fn get_per_cpu_base() -> usize {
    let base: usize;
    //println!("==================== get gs base a {:x} {:x}", msr::read_gs_base(), msr::read_kernel_gs_base());
    asm!("mov {base}, gs:0", base = out(reg)(base));
    //println!("==================== get gs base b");
    base
}
