use process::syscalls;

mod bochs;
mod raspi;

fn main() {
    println!("Hello from ab!");

    let platform = "raspi4";

    let mut display = if platform == "qemu" {
        let mut bochs = bochs::BochsAdapter::new().unwrap();
        bochs.set_mode(640, 480);
        bochs.fill();
    } else if platform == "raspi3" {
        let RPI_3_PERIPHERAL_BASE = 0x3f000000;
        let mut raspi = raspi::MailboxAdapter::new(RPI_3_PERIPHERAL_BASE);
        raspi.set_mode(640, 480);
        raspi.fill();
    } else if platform == "raspi4" {
        let RPI_4_PERIPHERAL_BASE = 0xfe000000;
        let mut raspi = raspi::MailboxAdapter::new(RPI_4_PERIPHERAL_BASE);
        //raspi.set_mode(640, 480);
        raspi.fill();
    } else {
        panic!("Unknown platform!");
    };

    syscalls::exit_process();
}