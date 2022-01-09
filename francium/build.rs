fn main() {
    let mut platform: String = "".to_string();
    let mut found_platform = false;

    for (var, _value) in std::env::vars() {
        if var.starts_with("CARGO_FEATURE_PLATFORM_") {
            println!("{}", var);
            if found_platform {
                panic!("Multiple platforms specified!");
            }

            found_platform = true;
            platform = var.as_str()[("CARGO_FEATURE_PLATFORM_".len())..].to_lowercase().clone();
        }
    }

    if platform == "" {
        panic!("No platform specified!");
    }

    let mut cc_builder = cc::Build::new();

    cc_builder.compiler("aarch64-none-elf-gcc")
        .no_default_flags(true)
        .file("src/kernel_entry.s")
        .file("src/memory.s")
        .file("src/arch/aarch64/context.s")
        .file("src/arch/aarch64/scheduler.s")
        .file("src/arch/aarch64/interrupt.s")
        .file("src/arch/aarch64/arch_timer.s");

    if platform == "virt" {
        cc_builder.file("src/stub_virt.s");
    } else if platform == "raspi4" {
        cc_builder.file("src/stub_raspi4.s");
    }

    cc_builder.compile("asm");

    println!("cargo:rustc-link-arg=-Tlink_{}.x", platform);
}