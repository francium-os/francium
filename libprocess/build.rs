fn main() {
    cc::Build::new()
        .compiler("aarch64-none-elf-gcc")
        .no_default_flags(true)
        .file("src/syscalls.s")
        .compile("asm");
}
