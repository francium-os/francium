use common::system_info::*;
use process::syscalls;

mod bochs;
#[cfg(target_arch = "aarch64")]
mod raspi;

fn main() {
    println!("Hello from disp!");

    let platform = if let SystemInfo::Platform(plat) =
        syscalls::get_system_info(SystemInfoType::Platform, 0).unwrap()
    {
        plat
    } else {
        panic!("GetSystemInfo failed");
    };

    if platform == Platform::Virt || platform == Platform::Pc {
        let mut bochs = bochs::BochsAdapter::new().unwrap();
        bochs.set_mode(640, 480);
        bochs.fill();
    } else if platform == Platform::Raspi3 {
        #[cfg(target_arch = "aarch64")]
        {
            let rpi_3_peripheral_base = 0x3f000000;
            let mut raspi = raspi::MailboxAdapter::new(rpi_3_peripheral_base);
            raspi.set_mode(640, 480);
            raspi.fill();
        }
    } else if platform == Platform::Raspi4 {
        #[cfg(target_arch = "aarch64")]
        {
            let rpi_4_peripheral_base = 0xfe000000;
            let mut raspi = raspi::MailboxAdapter::new(rpi_4_peripheral_base);
            raspi.set_mode(640, 480);
            raspi.fill();
        }
    } else {
        panic!("Unknown platform!");
    };

    syscalls::exit_process();
}
