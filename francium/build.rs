fn main() {
    cc::Build::new()
        .compiler("clang")
        .no_default_flags(true)
        .flag("--target=aarch64-none-unknown-elf") 
        .file("src/kernel_entry.s")
        .file("src/memory.s")
        .file("src/stub_virt.s")
        .file("src/arch/aarch64/context.s")
        .compile("asm");
}