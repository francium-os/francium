use spin::Mutex;
use super::font::FONT8X8;

pub enum EarlyFramebufferFormat {
    Rgb,
    Bgr
}

pub struct EarlyFramebuffer {
    pub framebuffer: &'static mut [u8],
    pub pixel_format: EarlyFramebufferFormat,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,

    // state
    pub x: usize,
    pub y: usize
}

impl EarlyFramebuffer {
    fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let off = (x + y * self.stride) * self.bytes_per_pixel;
                self.framebuffer[off] = 0;
                self.framebuffer[off + 1] = 0;
                self.framebuffer[off + 2] = 0;
            }
        }
    }

    fn print(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.x = 0;
                    self.y += 8;
                },
                _ => {
                    let font_entry = FONT8X8[c as usize];
                    for yy in 0..8 {
                        for xx in 0..8 {
                            let offset = (self.x + xx + (self.y + yy) * self.stride) * self.bytes_per_pixel;

                            if (font_entry[yy] & (1<<xx)) == (1<<xx) {
                                self.framebuffer[offset] = 0xff;
                                self.framebuffer[offset+2] = 0xff;
                                self.framebuffer[offset+1] = 0xff;
                            } else {
                                self.framebuffer[offset] = 0;
                                self.framebuffer[offset+1] = 0;
                                self.framebuffer[offset+2] = 0;
                            }
                        }
                    }
                    self.x += 8;
                }
            }
        }
    }
}

pub struct EarlyFramebufferLogger {
    fb: Mutex<EarlyFramebuffer>
}

impl EarlyFramebufferLogger {
    fn new(mut fb: EarlyFramebuffer) -> EarlyFramebufferLogger {
        EarlyFramebufferLogger {
            fb: Mutex::new(fb)
        }
    }
}

use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

impl<'a> core::fmt::Write for EarlyFramebuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

impl log::Log for EarlyFramebufferLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            core::fmt::Write::write_fmt(&mut *self.fb.lock(), format_args!("[{}] {}\n", record.level(), record.args())).unwrap();
        }
    }

    fn flush(&self) {}
}

static mut FB_LOGGER: Option<EarlyFramebufferLogger> = None;

pub fn init(logger: EarlyFramebuffer) -> Result<(), SetLoggerError> {
    unsafe {
        FB_LOGGER = Some(EarlyFramebufferLogger::new(logger));
        let res = log::set_logger(FB_LOGGER.as_ref().unwrap()).map(|()| log::set_max_level(LevelFilter::Trace));
        res
    }
}

pub fn clear_screen() {
    unsafe {
        FB_LOGGER.as_ref().unwrap().fb.lock().clear()
    }
}
