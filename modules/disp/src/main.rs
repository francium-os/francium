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

    let splash_rgb = include_bytes!("../splash.rgb");
    let splash_x: usize = 640;
    let splash_y: usize = 480;

    let (fb_width, fb_height, fb) = match platform {
        Platform::Virt | Platform::Pc => {
            let mut bochs = bochs::BochsAdapter::new().unwrap();
            bochs.set_mode(1024, 768);
            (1024, 768, bochs.get_framebuffer())
        },
        Platform::Raspi3 => {
            let rpi_3_peripheral_base = 0x3f000000;
            let mut raspi = raspi::MailboxAdapter::new(rpi_3_peripheral_base);
            raspi.set_mode(1920, 1080);
            (1920, 1080, raspi.get_framebuffer())
        },
        Platform::Raspi4 => {
            let rpi_4_peripheral_base = 0xfe000000;
            let mut raspi = raspi::MailboxAdapter::new(rpi_4_peripheral_base);
            raspi.set_mode(1920, 1080);
            (1920, 1080, raspi.get_framebuffer())
        },
        _ => {
            panic!("Unknown platform!")
        }
    };

    fb.fill(0x00000000);
    for y in 0..splash_y {
        for x in 0..splash_x {
            let splash_offset = (x + y * splash_x) * 3;
            fb[x + y*fb_width] = (splash_rgb[splash_offset] as u32) | (splash_rgb[splash_offset+1] as u32) << 8 | (splash_rgb[splash_offset + 2] as u32) << 16;
        }
    }

    // print platform
    
    syscalls::exit_process();
}
