use std::{os::unix::process::CommandExt, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=splash.png");
    Command::new("convert splash.png splash.rgb").arg("").exec();
}