use core::arch::asm;

const IA32_EFER: u32 = 0xc0000080;
const IA32_STAR: u32 = 0xc0000081;
const IA32_LSTAR: u32 = 0xc0000082;
const IA32_FMASK: u32 = 0xc0000084;
const IA32_FS_BASE: u32 = 0xc0000100;
/*const IA32_GS_BASE: u32 = 0xc0000101;
const IA32_KERNEL_GS_BASE: u32 = 0xc0000102;
const IA32_TSC_AUX: u32 = 0xc0000103;*/

pub unsafe fn read_msr(id: u32) -> usize {
    let mut value_low: u32;
    let mut value_high: u32;
    asm!("rdmsr", out ("eax") (value_low), out ("edx") (value_high), in ("ecx")(id));
    (value_low as usize) | ((value_high as usize) << 32)
}

pub unsafe fn write_msr(id: u32, value: usize) {
    let value_low: u32 = (value & 0xffffffff) as u32;
    let value_high: u32 = (value >> 32) as u32;
    asm!("wrmsr", in ("eax") (value_low), in ("edx") (value_high), in ("ecx")(id));
}

pub unsafe fn read_efer() -> usize {
    read_msr(IA32_EFER)
}

pub unsafe fn write_efer(efer: usize) {
    write_msr(IA32_EFER, efer)
}

pub unsafe fn read_star() -> usize {
    read_msr(IA32_STAR)
}

pub unsafe fn write_star(star: usize) {
    write_msr(IA32_STAR, star)
}

pub unsafe fn read_lstar() -> usize {
    read_msr(IA32_LSTAR)
}

pub unsafe fn write_lstar(lstar: usize) {
    write_msr(IA32_LSTAR, lstar)
}

pub unsafe fn read_fmask() -> usize {
    read_msr(IA32_FMASK)
}

pub unsafe fn write_fmask(fmask: usize) {
    write_msr(IA32_FMASK, fmask)
}

pub unsafe fn read_fs_base() -> usize {
    read_msr(IA32_FS_BASE)
}

pub unsafe fn write_fs_base(fs_base: usize) {
    write_msr(IA32_FS_BASE, fs_base)
}
