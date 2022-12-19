use crate::msr;

pub fn setup_syscall(syscall_handler: usize) {
    unsafe {
        // enable syscall instructions
        msr::write_efer(msr::read_efer() | (1 << 0));

        msr::write_fmask(1 << 9); // clear interrupt flag
                                  // kernel segment base = 0x08 (code seg = 0x08, stack seg = 0x10)
                                  // user segment base = 0x18 (32bit code seg = 0x18, stack seg = 0x20, 64bit code seg = 0x28)
        msr::write_star(0x08 << 32 | (0x18|3) << 48);
        msr::write_lstar(syscall_handler); // syscall handler location
    }
}
