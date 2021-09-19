fn main() {
    cc::Build::new()
        .compiler("clang")
        .no_default_flags(true)
        .flag("--target=aarch64-none-unknown-elf") 
        .file("src/syscalls.s")
        .compile("asm");
}