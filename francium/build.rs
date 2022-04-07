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

    println!("cargo:rustc-link-arg=-Ttargets/link_{}.x", platform);
}
