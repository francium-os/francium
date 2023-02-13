use std::path::Path;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = Path::new("target/release/");
    let kernel_path_str = "target/x86_64-unknown-none/release/francium_pc";
    let kernel = Path::new(&kernel_path_str);

    // create an UEFI disk image (optional)
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .unwrap();

    // create a BIOS disk image (optional)
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    /*println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());*/
}
