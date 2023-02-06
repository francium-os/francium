use process::syscalls;

mod bochs;

fn main() {
    println!("Hello from disp!");

    let mut bochs = bochs::BochsAdapter::new().unwrap();
    bochs.set_mode(640, 480);
    bochs.fill();

    syscalls::exit_process();
}
