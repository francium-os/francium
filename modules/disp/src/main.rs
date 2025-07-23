use common::system_info::*;
use francium_common::font;
use process::syscalls;

mod bochs;
mod platform;
#[cfg(target_arch = "aarch64")]
mod raspi;

fn print_string(fb: &mut [u32], pitch: usize, xx: usize, yy: usize, s: &str) -> usize {
    let mut offset: usize = 0;
    for c in s.chars() {
        print_char(fb, pitch, xx + offset, yy, c);
        offset += 8;
    }
    offset
}

fn print_char(fb: &mut [u32], pitch: usize, xx: usize, yy: usize, c: char) {
    let pixels = &font::FONT8X8[c as usize];
    for y in 0..8 {
        let row = pixels[y];
        for x in 0..8 {
            if (row & (1<<x)) != 0 {
                fb[(xx+x)+(yy+y)*pitch] = 0xffffffff;
            }
        }
    }
}

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

    let platform_name = platform.to_string();

    let (fb_width, fb_height, fb) = match platform {
        Platform::Virt | Platform::Pc => {
            if let Some(mut bochs) = bochs::BochsAdapter::new() {
                bochs.set_mode(1024, 768);
                (1024, 768, bochs.get_framebuffer())
            } else if let Some(platform) = platform::PlatformFramebuffer::new() {
                (platform.info.width, platform.info.height, platform.get_framebuffer())
            } else {
                panic!("No framebuffer found!!");
            }
        },
        #[cfg(target_arch = "aarch64")]
        Platform::Raspi3 => {
            let rpi_3_peripheral_base = 0x3f000000;
            let mut raspi = raspi::MailboxAdapter::new(rpi_3_peripheral_base);
            raspi.set_mode(1920, 1080);
            (1920, 1080, raspi.get_framebuffer())
        },
        #[cfg(target_arch = "aarch64")]
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
    let mut off = print_string(fb, fb_width, 0, 600, "Hello from platform '");
    off += print_string(fb, fb_width, off, 600, &platform_name);
    off += print_string(fb, fb_width, off, 600, "'");
    
    syscalls::exit_process();
}
