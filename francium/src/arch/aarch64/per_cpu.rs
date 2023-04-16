use core::arch::asm;

pub unsafe fn setup_per_cpu(base: usize) {
    asm!("msr tpidr_el1, {base}", base = in(reg)(base));
}

pub unsafe fn get_per_cpu_base() -> usize {
    let base: usize;
    asm!("mrs {base}, tpidr_el1", base = out(reg)(base));
    base
}