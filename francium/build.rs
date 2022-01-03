fn main() {
    cc::Build::new()
        .compiler("aarch64-none-elf-gcc")
        .no_default_flags(true)
        .file("src/kernel_entry.s")
        .file("src/memory.s")
        .file("src/stub_virt.s")
        .file("src/arch/aarch64/context.s")
        .file("src/arch/aarch64/scheduler.s")
        .file("src/arch/aarch64/interrupt.s")
        .file("src/arch/aarch64/arch_timer.s")
        .compile("asm");
}
