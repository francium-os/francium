use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let mut args = std::env::args().skip(1); // skip executable name

    let kernel_binary_path = {
        let path = PathBuf::from(args.next().unwrap());
        path.canonicalize().unwrap()
    };

    let bios = create_disk_images(&kernel_binary_path);
    println!("Created disk image at `{}`", bios.display());
}

pub fn create_disk_images(kernel_binary_path: &Path) -> PathBuf {
    let kernel_manifest_path =
        PathBuf::from("/home/stary/develop/osdev/francium/francium/Cargo.toml");
    let bootloader_manifest_path =
        bootloader_locator::locate_bootloader("bootloader", Some(&kernel_manifest_path)).unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest_path.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest_path);
    build_cmd.arg("--kernel-binary").arg(&kernel_binary_path);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest_path.parent().unwrap().join("target"));
    build_cmd
        .arg("--out-dir")
        .arg(kernel_binary_path.parent().unwrap());
    build_cmd.arg("--quiet");

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }

    let kernel_binary_name = kernel_binary_path.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_binary_path
        .parent()
        .unwrap()
        .join(format!("boot-bios-{}.img", kernel_binary_name));
    if !disk_image.exists() {
        panic!(
            "Disk image does not exist at {} after bootloader build",
            disk_image.display()
        );
    }
    disk_image
}
